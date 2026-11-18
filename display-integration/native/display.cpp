#include "fh2mod.h"
#include <windows.h>
#include <cstdint>
#include <cstring>
#include "hud_page.h"
static const FH2ModApi* api;
static FH2Mod* mod;
static bool map_on=true,speedo_on=true;
using IntGetter=int(*)(void*);
static IntGetter next_map_a,next_map_b,next_speedo;
static int map_a(void*){return map_on?3:0;}
static int map_b(void*){return map_on?3:0;}
static int speedo(void*){return speedo_on?1:0;}
static bool verify(unsigned char* game,unsigned rva){
    const unsigned char* at=game+rva;
    int32_t displacement;memcpy(&displacement,at+1,4);
    return at[0]==0xe8 && at+5+displacement==game+0x9247b0;
}
static DWORD WINAPI install(void*){
    auto game=static_cast<unsigned char*>(api->wait_for_code(120000));
    if(!game){api->log(mod,"HUD hooks: game code unavailable");return 0;}
    if(!verify(game,0x74d7a9)||!verify(game,0x74d7e6)||!verify(game,0x789d6c)){
        api->log(mod,"HUD option call guards mismatch; nothing hooked");return 0;
    }
    auto a=api->hook_call(mod,"Minimap option: initial",game+0x74d7a9,(void*)map_a,(void**)&next_map_a);
    auto b=api->hook_call(mod,"Minimap option: free roam",game+0x74d7e6,(void*)map_b,(void**)&next_map_b);
    auto c=api->hook_call(mod,"Speedometer option",game+0x789d6c,(void*)speedo,(void**)&next_speedo);
    if(!a||!b||!c||!api->enable_hooks(mod,nullptr)){api->log(mod,"HUD hook installation refused");return 0;}
    api->log(mod,"HUD display choices applied without changing saved game options");return 0;
}
FH2MOD_EXPORT int fh2mod_init(const FH2ModApi* value){
    if(value->api_version<3)return 1;api=value;
    FH2ModInfo info{};info.size=sizeof info;info.name="FHCM Display";info.version="1.0.0";info.platform_version="2.7.2";
    mod=api->register_mod(&info);if(!mod)return 1;
    wchar_t path[MAX_PATH];GetModuleFileNameW(nullptr,path,MAX_PATH);
    wchar_t* slash=wcsrchr(path,L'\\');if(!slash)return 1;
    wcscpy_s(slash+1,MAX_PATH-(slash+1-path),L"fh2-mods\\FHCMDisplay.ini");
    bool enabled=GetPrivateProfileIntW(L"Display",L"HUD",1,path)!=0;
    map_on=enabled&&GetPrivateProfileIntW(L"Display",L"Minimap",1,path)!=0;
    speedo_on=enabled&&GetPrivateProfileIntW(L"Display",L"Speedometer",1,path)!=0;
    if(!enabled){if(!api->set_file(mod,"UI/Scenes/Anthem/A_Hud.xaml",hidden_hud,sizeof hidden_hud-1))return 1;}
    else api->clear_file(mod,"UI/Scenes/Anthem/A_Hud.xaml");
    HANDLE thread=CreateThread(nullptr,0,install,nullptr,0,nullptr);if(!thread)return 1;CloseHandle(thread);return 0;
}
