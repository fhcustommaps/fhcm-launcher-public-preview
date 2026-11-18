#include "../FHCMPaths.h"
#include "../FHCMPaths.h"
// DevLab-only: use the game's native NoPVS initialization option for the skidpad.
#include <windows.h>
#include <stdint.h>
#include <string.h>
#include <wchar.h>
#include <stdio.h>
#include "fh2mod.h"
static const FH2ModApi *api;
static FH2Mod *mod;
static volatile LONG count;
static void enable_no_pvs(FH2Context *ctx,void *unused){
 ctx->rax=(ctx->rax&~(uint64_t)255)|1;
 InterlockedIncrement(&count);
}
static DWORD WINAPI worker(void *unused){
 unsigned char *base=(unsigned char*)api->wait_for_code(60000);
 static const unsigned char expected[]={0x88,0x47,0x74,0x41,0x0f,0xb6};
 if(!base||memcmp(base+0xf9e4e4,expected,sizeof(expected))){api->log(mod,"Build signature differs; NoPVS hook refused");return 0;}
 if(!api->hook_instruction(mod,"Native NoPVS scene initialization",base+0xf9e4e4,enable_no_pvs,NULL)||!api->enable_hooks(mod,NULL)){api->log(mod,"NoPVS hook unavailable");return 0;}
 api->log(mod,"Native NoPVS scene option enabled for this skidpad session");
 LONG last=0;
 for(;;){Sleep(2000);LONG now=count;if(now!=last){api->log(mod,"Scene initialization consumed NoPVS=true");last=now;}}
 return 0;
}
FH2MOD_EXPORT int fh2mod_init(const FH2ModApi *platform){
 if(!platform||platform->api_version<7||(_wcsicmp(platform->azimuth_dir,fhcm_w(L"."))&&_wcsicmp(platform->azimuth_dir,fhcm_w(L"..\\builds\\azimuth-upgrade-20261029\\runtime-1.0.4"))))return 1;
 if(GetFileAttributesW(fhcm_w(L"..\\skidpad-enabled.flag"))==INVALID_FILE_ATTRIBUTES)return 0;
 api=platform;FH2ModInfo info={sizeof(info),"DevLab Visibility","0.4.0",NULL,"2.7.2",0,"Skidpad-only native NoPVS initialization; used with map-wide floor residency"};
 mod=api->register_mod(&info);if(!mod)return 1;
 HANDLE t=CreateThread(NULL,0,worker,NULL,0,NULL);if(t)CloseHandle(t);return t?0:1;
}
