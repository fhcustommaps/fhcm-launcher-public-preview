#pragma once
#include <windows.h>
#include <wchar.h>
#include <stdio.h>
static const wchar_t* fhcm_w(const wchar_t* rel){ static __thread wchar_t paths[16][32768];static __thread unsigned n;wchar_t* b=paths[(n++)%16];GetModuleFileNameW(NULL,b,32768);wchar_t* s=wcsrchr(b,L'\\');if(s)*s=0;if(!wcscmp(rel,L"."))return b;wcscat(b,L"\\");wcscat(b,rel);return b;}
static const char* fhcm_a(const char* rel){static __thread char paths[16][32768];static __thread unsigned n;char* b=paths[(n++)%16];GetModuleFileNameA(NULL,b,32768);char* s=strrchr(b,'\\');if(s)*s=0;strcat(b,"\\");strcat(b,rel);return b;}
