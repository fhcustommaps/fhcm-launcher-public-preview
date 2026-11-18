#include "../FHCMPaths.h"
#include "../FHCMPaths.h"
/* Optional DevLab keyboard bridge for Windows.Gaming.Input. Physical pad
 * readings remain intact. Keyboard contributions require DevLab focus.
 * ABI reference: microsoft/windows-rs Windows/Gaming/Input/mod.rs. */
#include <windows.h>
#include <stdint.h>
#include <wchar.h>
#include <string.h>
#include "fh2mod.h"
typedef struct { uint64_t timestamp; uint32_t buttons,pad; double lt,rt,lx,ly,rx,ry; } Reading;
static const FH2ModApi *api; static FH2Mod *mod;
static HRESULT (WINAPI *read_original)(void*,Reading*);
static CRITICAL_SECTION lock;
static Reading keys;
__declspec(dllexport) volatile uint64_t DevLabInputDebug[8];
static HRESULT WINAPI read_keys(void *self,Reading *out){
 HRESULT hr=read_original(self,out);if(FAILED(hr)||!out)return hr;
 EnterCriticalSection(&lock);Reading k=keys;LeaveCriticalSection(&lock);
 out->buttons|=k.buttons;if(k.rt>out->rt)out->rt=k.rt;if(k.lt>out->lt)out->lt=k.lt;
 if(k.lx)out->lx=k.lx;memcpy((void*)&DevLabInputDebug[5],&out->rt,8);memcpy((void*)&DevLabInputDebug[6],&out->lt,8);memcpy((void*)&DevLabInputDebug[7],&out->lx,8);return hr;
}
static void release(void *p){if(p)((ULONG(WINAPI*)(void*))(*(void***)p)[2])(p);}
static DWORD WINAPI worker(void *unused){
 HMODULE c=LoadLibraryW(L"combase.dll");if(!c)return 0;
 HRESULT(WINAPI *init)(DWORD)=(void*)GetProcAddress(c,"RoInitialize");
 HRESULT(WINAPI *make)(PCWSTR,UINT32,void**)=(void*)GetProcAddress(c,"WindowsCreateString");
 HRESULT(WINAPI *factory)(void*,const GUID*,void**)=(void*)GetProcAddress(c,"RoGetActivationFactory");
 HRESULT(WINAPI *del)(void*)=(void*)GetProcAddress(c,"WindowsDeleteString");
 if(!init||!make||!factory||!del)return 0;init(1);
 GUID iid={0x8bbce529,0xd49c,0x39e9,{0x95,0x60,0xe4,0x7d,0xde,0x96,0xb7,0xc8}};
 void *str=NULL,*statics=NULL;const wchar_t *name=L"Windows.Gaming.Input.Gamepad";
 if(FAILED(make(name,(UINT32)wcslen(name),&str)))return 0;
 HRESULT hr=factory(str,&iid,&statics);del(str);if(FAILED(hr))return 0;
 FH2Hook *hook=NULL;
 for(int attempt=0;attempt<60&&!hook;attempt++){
  void *list=NULL,*pad=NULL;
  if(SUCCEEDED(((HRESULT(WINAPI*)(void*,void**))(*(void***)statics)[10])(statics,&list))&&list){
   UINT32 count=0;((HRESULT(WINAPI*)(void*,UINT32*))(*(void***)list)[7])(list,&count);
   if(count&&SUCCEEDED(((HRESULT(WINAPI*)(void*,UINT32,void**))(*(void***)list)[6])(list,0,&pad))&&pad){
    hook=api->hook_function(mod,"Focused keyboard contribution to gamepad reading",(*(void***)pad)[8],read_keys,(void**)&read_original);release(pad);
   }
   release(list);
  }
  if(!hook)Sleep(500);
 }
 release(statics);if(!hook||!api->enable_hooks(mod,hook))return 0;
 api->log(mod,"Keyboard bridge ready: Enter A, Backspace B, P pause, WASD drive; F9 gentle/F6 stronger bounded throttle; J/L bounded steering; K bounded brake; F8 cancel");
 int vk[]={13,8,80,38,40,37,39,32,88,89,219,221};
 unsigned bits[]={4,8,1,64,128,256,512,4,16,32,1024,2048};
 ULONGLONG pulse[12]={0},testUntil=0,steerUntil=0,brakeUntil=0;double steerPulse=0,testThrottle=.25;int wasF9=0,wasF6=0;
 for(;;){
  DWORD pid=0;GetWindowThreadProcessId(GetForegroundWindow(),&pid);
  int active=pid==GetCurrentProcessId()&&GetFileAttributesW(fhcm_w(L"..\\keyboard-bridge.flag"))!=INVALID_FILE_ATTRIBUTES;
  ULONGLONG now=GetTickCount64();Reading k={0};
  for(int i=0;i<12;i++){
   SHORT state=GetAsyncKeyState(vk[i]);if(active&&(state&1))pulse[i]=now+180;
   if(active&&((state&0x8000)||now<pulse[i]))k.buttons|=bits[i];
   if(!active)pulse[i]=0;
  }
  SHORT f9=GetAsyncKeyState(VK_F9);int held=(f9&0x8000)!=0;
  if(active&&((f9&1)||(held&&!wasF9))){testUntil=now+2000;testThrottle=.25;}wasF9=held;
  SHORT f6=GetAsyncKeyState(VK_F6);int held6=(f6&0x8000)!=0;
  if(active&&((f6&1)||(held6&&!wasF6))){testUntil=now+2000;testThrottle=.7;}wasF6=held6;
  if(active&&(GetAsyncKeyState('J')&1)){steerUntil=now+2000;steerPulse=-.55;}
  if(active&&(GetAsyncKeyState('L')&1)){steerUntil=now+2000;steerPulse=.55;}
  if(active&&(GetAsyncKeyState('K')&1)){brakeUntil=now+2000;testUntil=0;steerUntil=0;}
  if(!active||(GetAsyncKeyState(VK_F8)&0x8001))testUntil=steerUntil=brakeUntil=0;
  if(active){
   k.rt=(GetAsyncKeyState('W')&0x8000)?1:now<testUntil?testThrottle:0;
   k.lt=((GetAsyncKeyState('S')&0x8000)||now<brakeUntil)?1:0;
   k.lx=((GetAsyncKeyState('D')&0x8000)?1:0)-((GetAsyncKeyState('A')&0x8000)?1:0);
   if(!k.lx&&now<steerUntil)k.lx=steerPulse;
  }
  DevLabInputDebug[0]=active;DevLabInputDebug[1]=now;DevLabInputDebug[2]=testUntil;DevLabInputDebug[3]=steerUntil;memcpy((void*)&DevLabInputDebug[4],&k.rt,8);
  EnterCriticalSection(&lock);keys=k;LeaveCriticalSection(&lock);Sleep(8);
 }
 return 0;
}
FH2MOD_EXPORT int fh2mod_init(const FH2ModApi *p){
 if(!p||p->api_version<7||_wcsicmp(p->azimuth_dir,fhcm_w(L".")))return 0;
 if(GetFileAttributesW(fhcm_w(L"..\\keyboard-bridge.flag"))==INVALID_FILE_ATTRIBUTES)return 0;
 api=p;FH2ModInfo info={sizeof(info),"DevLab Keyboard Bridge","0.3.0",NULL,"2.7.2",0,"Focused keyboard input for autonomous DevLab testing"};
 mod=api->register_mod(&info);if(!mod)return 1;InitializeCriticalSection(&lock);
 HANDLE t=CreateThread(NULL,0,worker,NULL,0,NULL);if(t)CloseHandle(t);return t?0:1;
}
