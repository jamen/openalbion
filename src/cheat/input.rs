use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{BOOL,LPARAM};
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::winnt::{HANDLE,PROCESS_ALL_ACCESS};
use winapi::um::winuser::{ShowWindow,EnumWindows,GetWindowThreadProcessId,WNDENUMPROC};
use winapi::um::tlhelp32::{PROCESSENTRY32,CreateToolhelp32Snapshot,TH32CS_SNAPPROCESS,Process32First,Process32Next};
use winapi::um::processthreadsapi::{OpenProcess,GetProcessId};
use winapi::um::handleapi::CloseHandle;
use winapi::um::errhandlingapi::{SetLastError,GetLastError};
use std::ffi::CString;
use std::thread::sleep;
use std::time;
use std::ptr::{null,null_mut};
use std::mem;

struct EnumData {
    dw_process_id: u32,
    fable_hwnd: HWND,
}

extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let mut enum_data = &mut lparam as &mut EnumData;

    let mut dw_process_id: u32 = 0;

    unsafe { GetWindowThreadProcessId(hwnd, &mut dw_process_id) };

    if (enum_data.dw_process_id == dw_process_id) {
        enum_data.fable_hwnd = hwnd;

        SetLastError(ERROR_SUCCESS);

        return 0;
    }

    return 1;
}

fn input() {
    let mut entry: PROCESSENTRY32 = PROCESSENTRY32::default();
    entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    let snapshot: HANDLE = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    let mut fable_hwnd: HANDLE = null_mut();

    if unsafe { Process32First(snapshot, &mut entry) == 1 } {
        while unsafe { Process32Next(snapshot, &mut entry) == 1 } {
            let szExeFile: [u8; 260] = unsafe { mem::transmute(entry.szExeFile) };
            let szExeFile: String = String::from_utf8(szExeFile[0..9].to_vec()).unwrap();

            if szExeFile == "Fable.exe" {
                let hProcess: HANDLE = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, entry.th32ProcessID) };

                let mut enum_data = EnumData {
                    dw_process_id: hProcess,
                    fable_hwnd: null_mut(),
                };

                if (EnumWindows(Some(enum_proc), &mut enum_data as LPARAM) == 1 && GetLastError() == ERROR_SUCCESS) {
                    fable_hwnd = enum_data.fable_hwnd;
                }

                unsafe { CloseHandle(hProcess) };

                break
            }
        }

        unsafe { CloseHandle(snapshot) };
    }
    // let fable_wnd: HWND = 0xff0b86 as HWND;

    println!("fable_hwnd {:?}", fable_hwnd);

    // unsafe { ShowWindow(fable_wnd, 9); }

    // let mut press_amount = 50;

    // while press_amount > 0 {
    //     unsafe { SendMessageA(fable_wnd, WM_KEYDOWN, 0x57, 1); } // press W

    //     let duration = time::Duration::from_millis(1000);

    //     sleep(duration);

    //     unsafe { SendMessageA(fable_wnd, WM_KEYUP, 0x57, 1); } // release W

    //     press_amount -= 1;
    // }
}

//
// From fabletlcmod.com:
//
// sub_C05FD0 - zlib crc32 function
// sub_CBFB7D - main scripts.bin script parser/engine (over 80kb of code in a SINGLE function...)
// sub_CD52D0 - compiled script registering process( best place to add our own stuff into)
//
// sub_5D1FC - hero stats function PC (maybe)
// sub_5CE0E6 - hero stats display UI (maybe)
// sub_409730 - Enum profile and game save files
// sub_99AD80 - open file handler
// sub_99A6A0 - open file
// sub_4A21F0 - fablesav parser/loader
// sub_40D350 - main profile loader, calls below
// sub_40BCA0 - profile parser
// sub_9F1D20 - boot.ini loader
//
// sub_CE6CF0 - S_GF (Register)
// sub_CE75B0 - S_GF (Main)
// sub_CE7640
// sub_CE7650
// sub_CE7670 - S_GF (Story Flow)
// sub_CEF3B0 - S_GF (NewQuestCard)
// sub_CEF550 - S_GF (Barrowfield)
// sub_CEF8E0 - S_GF (Save)
// sub_CEF950 - S_GF (Game Flow)
// sub_CEF9A0
// sub_CEFA00
// sub_CEFA20 - S_GFA (Main)
// sub_CEFAB0
// sub_CEFAC0 - S_GFA (Tutorial)
// sub_CEFCC0 - S_GFA (MultiCheck)
// sub_CEFFB0 - S_GFA (HealthCheck)
// sub_CF0180 - S_GFA (WillCheck)
// sub_CF02A0 - S_GFA (RenownCheck)
// sub_CF0540
// sub_CF0560 - S_GFA (Stats)
// sub_CF0640 - S_GFA (Gameflow Assistant)
//
//
//
// DemonDoor_Start              .text 00E6E2E0 0000008B R . . . . . .
// DemonDoors                   .text 00E75590 00000036 R . . . . . .
// Fisticuffs_Complete          .text 00E82190 00000083 R . . . . . .
// Fisticuffs_Create            .text 00E82220 000002AB R . . . . . .
// Fisticuffs_Crowd             .text 00E83A20 0000019B R . . . . . .
// Fisticuffs_Cutscene          .text 00E8A1D0 000000C8 R . . . . . .
// Fisticuffs_Introduced        .text 00E84B50 00000021 R . . . . . .
// Fisticuffs_KHG_Win           .text 00E8B020 000006D4 R . . . . . .
// Fisticuffs_Main_             .text 00E84B80 0000226C R . . . . . .
// Fisticuffs_NPC_Create        .text 00E8A2B0 00000CBF R . . . . . .
// Fisticuffs_Tyler             .text 00E87030 00003107 R . . . . . .
// PAAWB_NPC                    .text 00EC12E0 000000F1 R . . . . . . (Picnic Area After Wasp Battle script)
// RPS_BANDIT_ARCHER            .text 00EC32F0 000007BB R . . . . . . (Random Population Sim Script Functions)
// RPS_BANDIT_GRUNT             .text 00EC3AD0 000000D6 R . . . . . .
// RPS_Exit                     .text 00EC1CD0 0000021C R . . . . . .
// RPS_Main                     .text 00EC1850 0000008B R . . . . . .
// RPS_Morality                 .text 00EC1EF0 00000344 R . . . . . .
// RPS_NPC                      .text 00EC1910 0000024C R . . . . . .
// RPS_Spawn                    .text 00EC29F0 00000739 R . . . . . .
// RPS_TRADER                   .text 00EC3130 000000EB R . . . . . .
// SM_Main                      .text 00ED3A90 0000008B R . . . . . . (Statue Master Functions)
// SM_Markers                   .text 00ED3B30 0000057A R . . . . . .
// S_QGT                        .text 00D50600 00000047 R . . . . . .
// S_QGT_GuildTrain_Main        .text 00D3BB50 0000008B R . . . . . .
// S_VPAAWB                     .text 00EC1780 00000036 R . . . . . .
// S_VPAAWB_Main                .text 00EC1240 0000008B R . . . . . .
// S_VRPS                       .text 00EC3BC0 00000039 R . . . . . .
// S_VSM                        .text 00ED4A40 00000036 R . . . . . .
// ScriptMain                   .text 00CDE2F0 00000036 R . . . . . .
// Script_Global                .text 00CE19A0 00000036 R . . . . . .
// TH_Cutscene_Triggers         .text 00EDED10 00000583 R . . . . . . (Wandering heroes Script Functions)
// TH_Main                      .text 00EDEC70 0000008B R . . . . . .
// TH_Scenes                    .text 00EE1DC0 00000077 R . . . . . .
//
//
//
// Dev Console = 0x009ED190
//
// This has been disabled in some way.  As far as I can tell, the console is intact, all the routines for initializing it, and it's graphics, etc, exist. (It's all running too, it's just never enabled by the game.)
//
// Decrease Will:
// 0x0057B1F1 - (add [esi+58h], eax)
//
// This could be used to remove magic cost, or, to create a multiplier to increase the cost.
//
// Increase\Decrease Gold (Shops, maybe more..)
// 0x0057B338 - (mov [esi+3Ch], eax)
//
// Static References:
//
// GameDirectory = 0x013BCA10
// HInstance = 0x013BD6EC
//
// CThingManager = 0x013B8A1C
// GraphicDataBank = 0x013B8A08
// MeshDataBank = 0x013B8A04
// QuestManager = 0x013B89FC
// CGameJoystickManager = 0x013B89A0
// CStreamingFontBank = 0x013B8998
// CThingObjectDef 0x013B8C14
// CInventoryItemDef 0x013B8C18
// CUserProfileManager = 0x013B7D4C
// CGraphicBankManager = 0x013B837C
// CShaderRenderManager = 0x013B8380
// CRenderManager = 0x013B8384
// CInputManager = 0x013B8388
// CFontManager = 0x013B838C
// CDisplayManager = 0x013B8390
// CSoundManager = 0x013B8394
// CGame = 0x013B83D0
// CMainGameComponent = 0x013B86A0
// CManager@NUISystem = 0x013B8710
// CPlayerDef = 0x013B878C
// CPlayerGUI = 0x013B8790
// CGameDefinitionManager = 0x013B879C
// CEngineManager = 0x013BA854
// CTCAICreatureWillPowerIndicator = 0x013BA89C
// CCameraModeDef = 0x013BA8D8
// CSkeletalMorphResourceManager = 0x013BAB10
//
// I haven't verified all of these, they could be static, or the values could be temporarily stored there. (I'll have to keep checking them, and make sure they always stay the same.)
//
// I hit the damn static lottery. :)
//
//
// These are the layouts of class instances mapped in memory.
//
// CThingManager:
//
// Base = CThingManager (VFTable: 0x01245C44)
// Base + 1Ch = CMainGameComponent
// Base + 20h = CGameDefinitionManager
// Base + 24h = CWorld
// Base + 28h = CWorldMap
// Base + 30h = CPlayerManager
// Base + 8Ch = Unknown
//
// CPlayerManager:
//
// Base = CPlayerManager (VFTable: 0x01231CD0)
// Base + 0Ch = CPlayer
// Base + 10h = Unknown
// Base + 1Ch = Unknown
//
// CPlayer:
//
// Base = CPlayer (VFTable: 0x01231CC4)
// Base + 0Ch = CGamePlayerInterface
// Base + 34h = CIntelligentPointer@VCThingPlayerCreature
//
// CIntelligentPointer@VCThingPlayerCreature:
//
// Base + 4h = CThingPlayerCreature
//
// CThingPlayerCreature:
//
// Base = CThingPlayerCreature (VFTable: 0x012457FC)
// Base + 0B0h = Max Health (Float)
// Base + 0B4h = Current Health (Float)
//
// CTCHeroStats:
//
// Base = CTCHeroStats (VFTable: 0x0124F70C)
// Base + 4h = CThingPlayerCreature (VFTable: 0x012457FC)
// Base + 38h = Unknown
// Base + 3Ch = Current Gold
// Base + 40h = Highest Amount of Money Ever Had
// Base + 48h = Total Money Acquired
// Base + 4Ch = Total Money Spent
// Base + 58h = Current Will
// Base + 5Ch = Max Will
// Base + 70h = Renown
// Base + FCh = Total Fines
//
// CSystemManager:
//
// Base + 58h = CInputManager
// Base + 60h = CDisplayManager
// Base + 7Ch = CSoundManager
// Base + 84h = CFontManager
//
// CDisplayManager:
//
// Base + 8h = CRenderManager
//
// CDrawPerceivers@NPlayerGui:
//
// Base + 38h = Perceiver Count
//
// CGameCameraManager:
//
// Base + 114h = Unknown
// Base + 118h = Unknown
// Base + 128h = Unknown
//
// CInventoryItemDef:
//
// Base + 4h = Unknown
//
//
//
// 0x99EBF0 is a string constructor. The game uses something called "CCharString" likely to provide wrapper functions and such. In the following image, Mac code (with built-in labels) is left, with the corresponding PC code is on the right. You can see the 99ebf0 call corresponds to a Mac call of CCharString's constructor (taking a char* and its length). This code is from the Necropolis tablet scripting.
// https://i.imgur.com/y4yKcRN.png (Please ignore that I've accidentally named the destructor the same thing - 99ebf0 is the constructor, and 99eae0 is the destructor.)
//
// Similarly, 0x9ed190 *is* related to the console - this is CConsole::Initialise(CConsole*, char, EInputKey, CFontBank*).
//
// if you change the byte 01375741 aka Fable.exe+F75741 from one to zero, you can leave quest zones instead of being forced to reload
// Debug profiles:
//
// the routine at Fable.exe+7030 is labeled CUserProfileManager::IsDebugProfile. Bytepatch information:
//
// original text
// xor al, al
// retn
// original dump
// 32 c0 c3
// new text
// mov al, 1
// retn
// new dump
// b0 01 c3
//
// This bytepatch fools the game into believing you are playing on a "debug profile". It has the currently noted effects:
// 1. you may world save whenever you want
// 2. all empty save slots except the first one are hidden
// 3. saves are saved in "Save04" instead of "Manual - Save04"
// World saving inside a quest seems to work fine - you are put back in the quest at the same state when you load. Needs a lot more testing though.
//
//
//
// Father / Hero - Interactions
//
// Sub                  Location            Purpose:
// .text:00DB8BFD                           ;Initial Check routine.
// .text:00DB8BFD       00DB8C56            mov     ecx, [eax+54h] ;read our good deed variable (you can change to int and nop rest.
// .text:00DB8BFD       00DB8C5B            jnz     loc_DB8D39 ; after a test (good deed) this calls routine
// .text:00DB8BFD       00DB8C61            mov     edx, [eax+58h] ;our money variable
// .text:00DB8BFD       00DB8C66            jnz     loc_DB8D39 ; after a money test this calls our routine
// .text:00DB8BFD       00DB8CA7            jmp     short loc_DB8CAB ;call you should do more speech if nothing has changed
//
// .text:00DB8CAB                           ;This loads the text/speech for the father "TEXT_QST_048_DAD_DONE_NOTHING_YET"
//
// .text:00DB8D39                           ;called by both money and good deeds/ at the end calls reward routine
// .text:00DB8D39       00DB8D51            mov     ecx, [esi+1Ch] ;read good deeds?
// .text:00DB8D39       00DB8D54            mov     eax, [esi+14h] ; read money?
// .text:00DB8D39       00DB8D57            mov     eax, [eax+54h];read our good deed variable
// .text:00DB8D39       00DB8D5A            sub     eax, ecx ; subtract deeds from gold?
// .text:00DB8D39       00DB8D5C            add     ecx, eax ; give back the difference to gold earned
// .text:00DB8D39       00DB8D77            jnz     loc_DB8E38 ; if money != deeds call process that calls bad deed reward
// .text:00DB8D39       00DB8DB5            jmp     short loc_DB8DB9 ; call good reward sequence
//
// .text:00DB8DB9                           ;call good reward sequence TEXT_QST_048_DAD_GIVE_REWARD_JUST_GOOD
// .text:00DB8DB9       00DB8DC7            jz      loc_DB8EEA ; if you have enough money call enough money speech
//
// .text:00DB8E38                           ; leads to Bad Reward sequence
// .text:00DB8E38       00DB8E74            jmp     short loc_DB8E78 ; call bad reward
//
// .text:00DB8E78                           ;calls for TEXT_QST_048_DAD_GIVE_REWARD_PART_BAD to be said / displayed
// .text:00DB8E78       00DB8E86            jz      short loc_DB8EEA ; if you have enough money call enough money speech
//
// .text:00DB8EEA                           ;check for chocolate
//
// Barrel Man:
//
// Sub                  Location            Purpose:
// text:00DB4FC0                            Load Quest related data icon etc
// text:00DB4FC0        00DB4FE3            push    offset aHud_clock_icon ; "HUD_CLOCK_ICON" ;display icon
