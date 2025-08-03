#include<pthread.h>
#include<stdio.h>
#include<stdlib.h>
#include<string.h>
#include<unistd.h>
#include<linux/input.h>
#include<linux/types.h>
#ifdef DEBUG
#define dbg(...) {\
    fprintf(stderr,##__VA_ARGS__);\
    fflush(stderr);\
}
#else
#define dbg(...)
#endif
enum KEY_STATES{
    KEY_STATE_RELEASED,
    KEY_STATE_PRESSED,
    KEY_STATE_REPEATED,
    KEY_STATE_DELAYING
};
__s8 keystate[1<<16];
struct EvStack{
    struct input_event evs[8];
    __s8 n;
};
#define DEBOUNCE_DELAY 12000 //delay time for release (us)
pthread_mutex_t mutex=PTHREAD_MUTEX_INITIALIZER;
struct __key_release_delay_arg{
    struct EvStack evstack;
    __u16 code;
};
void *key_release_delay(void *arg){
    struct __key_release_delay_arg *args=(struct __key_release_delay_arg*)arg;
    struct EvStack *events=&args->evstack;
    __u16 keycode=args->code;
    usleep(DEBOUNCE_DELAY);
    for(int i=0;i<events->n;i++){
        events->evs[i].input_event_usec+=DEBOUNCE_DELAY;
        while(events->evs[i].input_event_usec>=1000000){
            events->evs[i].input_event_usec-=1000000;
            events->evs[i].input_event_sec++;
        }
    }
    pthread_mutex_lock(&mutex);
    if(keystate[keycode]==3){
        keystate[keycode]=0;
        fwrite(events->evs,sizeof(struct input_event)*events->n,1,stdout);
        dbg("\t\x1b[1;35mAfter delay...\x1b[0m\n");
        for(int i=0;i<events->n;i++)
            dbg("\ttime %ld.%.6ld, type %d, code %d, value %d\n",
                events->evs[i].input_event_sec,
                events->evs[i].input_event_usec,
                events->evs[i].type,
                events->evs[i].code,
                events->evs[i].value);
    }
    pthread_mutex_unlock(&mutex);
    free(args);
    return NULL;
}
int main(){
#ifdef DEBUG
    if(!freopen("./keyboard-debouncer.log","w",stderr)){
        perror("freopen() failed");
        return EXIT_FAILURE;
    }
#endif
    memset(keystate,0,sizeof(keystate));
    setbuf(stdin,NULL);
    setbuf(stdout,NULL);
    struct EvStack events;
    events.n=0;
    while(fread(events.evs+events.n,sizeof(struct input_event),1,stdin)==1){
        events.n++;
        if(events.evs[events.n-1].type||events.evs[events.n-1].code||events.evs[events.n-1].value)
            continue;
        __s8 detached=0;
        for(int i=0;i<events.n;i++){
            if(events.evs[i].type==EV_KEY&&events.evs[1].value==0){
                dbg("\ttime %ld.%.6ld, type %d, code %d, value %d\n",
                    events.evs[i].input_event_sec,
                    events.evs[i].input_event_usec,
                    events.evs[i].type,
                    events.evs[i].code,
                    events.evs[i].value);
                dbg("\t\x1b[1;34mWaiting for delay...\x1b[0m\n");
                pthread_mutex_lock(&mutex);
                keystate[events.evs[i].code]=KEY_STATE_DELAYING;
                pthread_mutex_unlock(&mutex);
                struct __key_release_delay_arg *events_copy=malloc(sizeof(struct __key_release_delay_arg));
                if(events_copy){
                    memcpy(&events_copy->evstack,&events,sizeof(struct EvStack));
                    events_copy->code=events.evs[i].code;
                    pthread_t th;
                    if(pthread_create(&th,NULL,key_release_delay,events_copy)==0)
                        pthread_detach(th);
                    detached=1;
                    break;
                }else{
                    pthread_mutex_lock(&mutex);
                    keystate[events.evs[i].code]=KEY_STATE_RELEASED;
                    pthread_mutex_unlock(&mutex);
                }
            }else{
                dbg("time %ld.%.6ld, type %d, code %d, value %d\n",
                    events.evs[i].input_event_sec,
                    events.evs[i].input_event_usec,
                    events.evs[i].type,
                    events.evs[i].code,
                    events.evs[i].value);
                if(events.evs[i].type==EV_KEY)
                    keystate[events.evs[i].code]=events.evs[i].value;
            }
        }
        if(!detached)
            fwrite(&events.evs,sizeof(struct input_event)*events.n,1,stdout);
        events.n=0;
    }
    return 0;
}
