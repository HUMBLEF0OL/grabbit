; Mister Download Manager's Windows installer (Inno Setup 6). Built by tools/installer/build.ps1,
; which passes /DAppVersion=... and, when a certificate is set up, /DSign with /Smdmsign=...
;
; Install for the current user (no administrator rights, %LOCALAPPDATA%\Programs) or for
; everyone (Program Files); the wizard asks. Tasks: browser integration (native messaging host
; for Chrome, Edge, Brave and Firefox), .torrent files and magnet links, start with Windows, a
; desktop shortcut. A running MDM is asked to save and quit first (/exit). Uninstalling asks
; whether to keep the download list and settings; removing them also removes MDM's sign-ins from
; Windows Credential Manager (grabbit --forget-sign-ins).

#ifndef AppVersion
  #define AppVersion "1.0.1"
#endif
#define AppName "Grabbit"
#define AppExe "grabbit.exe"
#define HostExe "grabbit-nmhost.exe"
#define Root "..\.."
#define Bin Root + "\target\release"

[Setup]
; Never change AppId: upgrades and the uninstaller find the installation by it.
AppId={{667DB4D8-32C2-4487-A0E5-5825BF29778C}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=HUMBLEF0OL
AppPublisherURL=https://github.com/HUMBLEF0OL/grabbit
AppSupportURL=https://github.com/HUMBLEF0OL/grabbit/issues
AppUpdatesURL=https://github.com/HUMBLEF0OL/grabbit/releases
AppCopyright=Copyright (C) 2026 HUMBLEF0OL and contributors; portions (C) 2026 Ahmed Nefzaoui
DefaultDirName={autopf}\{#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
; The Run entry is the installing user's, also when installing for everyone.
UsedUserAreasWarning=no
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
WizardStyle=modern
SetupIconFile={#Root}\crates\app\assets\icons\app\mdm.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
OutputDir={#Root}\target\dist
OutputBaseFilename=Grabbit-{#AppVersion}-setup
; A 64 MB dictionary (#25): an 8 MB one (lzma2/max) only finds what repeats close by in the
; solid archive. A few MB smaller; building needs about 750 MB of memory, installing 64 MB.
Compression=lzma2/ultra64
SolidCompression=yes
ChangesAssociations=yes
; A running copy is asked to quit by [Code] (it holds the mutex Grabbit.Running);
; AppMutex isn't used because it would refuse to start before that. Restart Manager is the
; fallback for anything else holding the files (a browser running the host).
CloseApplications=yes
VersionInfoVersion={#AppVersion}
VersionInfoCompany=HUMBLEF0OL
VersionInfoDescription={#AppName} setup
VersionInfoProductName={#AppName}
#ifdef Sign
SignTool=mdmsign
SignedUninstaller=yes
  #define SignFlag "signonce"
#else
  #define SignFlag ""
#endif

; English only (the other languages live on the archive/multiplatform-i18n branch).
[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
TaskBrowser=Set up &browser integration (Chrome, Edge, Brave, Firefox)
TaskTorrents=Open .&torrent files and magnet links with {#AppName}
TaskStartup=Start {#AppName} when &Windows starts
TaskGroupOther=Other:
Launch=Start {#AppName}
KeepData=Also delete your download list and settings?%n%nYour downloaded files are not touched either way.

[Tasks]
Name: "browser"; Description: "{cm:TaskBrowser}"
Name: "torrents"; Description: "{cm:TaskTorrents}"
Name: "startup"; Description: "{cm:TaskStartup}"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:TaskGroupOther}"; Flags: unchecked

[Files]
Source: "{#Bin}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion {#SignFlag}
Source: "{#Bin}\{#HostExe}"; DestDir: "{app}"; Flags: ignoreversion {#SignFlag}
Source: "{#Bin}\grabbit-cli.exe"; DestDir: "{app}"; Flags: ignoreversion {#SignFlag}
Source: "{#Bin}\grabbit-mcp.exe"; DestDir: "{app}"; Flags: ignoreversion {#SignFlag}
; The extension, for "Load unpacked" (Chromium) and about:debugging (Firefox) until it is in
; the stores.
Source: "{#Root}\target\extension\chromium\*"; DestDir: "{app}\extension\chromium"; Flags: ignoreversion recursesubdirs
Source: "{#Root}\target\extension\mdm-firefox.xpi"; DestDir: "{app}\extension"; Flags: ignoreversion
Source: "{#Root}\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "{#Root}\ATTRIBUTION.md"; DestDir: "{app}"; DestName: "ATTRIBUTION.txt"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
; Start with Windows: the same value Options -> "Launch on startup" writes and reads, with the
; argument that starts MDM in the notification area (#226; win32.rs startup_command).
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Grabbit"; ValueData: """{app}\{#AppExe}"" /startup"; Flags: uninsdeletevalue; Tasks: startup
; The "torrents" task, ticked or not: Torrent settings' switch on the app's first start that hasn't
; decided it (upstream issue 1058: it then also decides whether the browser hands .torrent
; downloads over; settings.rs settle_torrent_handler, win32.rs install_torrents). Only read, once.
Root: HKCU; Subkey: "Software\Grabbit"; ValueType: string; ValueName: "InstallTorrents"; ValueData: "1"; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: torrents
Root: HKCU; Subkey: "Software\Grabbit"; ValueType: string; ValueName: "InstallTorrents"; ValueData: "0"; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: not torrents
; .torrent files and magnet links (HKA: the user's or everyone's classes, as installed).
Root: HKA; Subkey: "Software\Classes\Grabbit.torrent"; ValueType: string; ValueName: ""; ValueData: "Torrent file"; Flags: uninsdeletekey; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\Grabbit.torrent\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#AppExe},0"; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\Grabbit.torrent\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\.torrent"; ValueType: string; ValueName: ""; ValueData: "Grabbit.torrent"; Flags: uninsdeletekeyifempty; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\.torrent\OpenWithProgids"; ValueType: string; ValueName: "Grabbit.torrent"; ValueData: ""; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\magnet"; ValueType: string; ValueName: ""; ValueData: "URL:Magnet link"; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\magnet"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\magnet\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#AppExe},0"; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\magnet\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Tasks: torrents
; Windows' Default apps (upstream issue 1091): Grabbit listed for MAGNET and .torrent, where the
; user picks it (a choice recorded there for another program stays first, and no program may
; change it). The same keys and names as win32.rs set_torrent_handlers; ApplicationName matches
; the RegisteredApplications value's name, and each association has a Grabbit-only ProgID.
Root: HKA; Subkey: "Software\Classes\Grabbit.Magnet"; ValueType: string; ValueName: ""; ValueData: "URL:Magnet link"; Flags: uninsdeletekey; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\Grabbit.Magnet"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\Grabbit.Magnet\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"",0"; Tasks: torrents
Root: HKA; Subkey: "Software\Classes\Grabbit.Magnet\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Tasks: torrents
Root: HKA; Subkey: "Software\Grabbit\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#AppName}"; Flags: uninsdeletekey; Tasks: torrents
Root: HKA; Subkey: "Software\Grabbit\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Downloads files, videos and torrents"; Tasks: torrents
Root: HKA; Subkey: "Software\Grabbit\Capabilities"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: """{app}\{#AppExe}"",0"; Tasks: torrents
Root: HKA; Subkey: "Software\Grabbit\Capabilities\FileAssociations"; ValueType: string; ValueName: ".torrent"; ValueData: "Grabbit.Torrent"; Tasks: torrents
Root: HKA; Subkey: "Software\Grabbit\Capabilities\UrlAssociations"; ValueType: string; ValueName: "magnet"; ValueData: "Grabbit.Magnet"; Tasks: torrents
Root: HKA; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "{#AppName}"; ValueData: "Software\Grabbit\Capabilities"; Flags: uninsdeletevalue; Tasks: torrents

[Run]
Filename: "{app}\{#HostExe}"; Parameters: "--register"; Flags: runhidden; Tasks: browser; Check: not IsAdminInstallMode
Filename: "{app}\{#HostExe}"; Parameters: "--register --all-users"; Flags: runhidden; Tasks: browser; Check: IsAdminInstallMode
Filename: "{app}\{#AppExe}"; Description: "{cm:Launch}"; Flags: nowait postinstall skipifsilent
; Update now in MDM runs this setup silently with /relaunch=1: MDM starts again once installed,
; as the user (not as the administrator an all-users update asked for).
Filename: "{app}\{#AppExe}"; Parameters: "/updated"; Flags: nowait runasoriginaluser; Check: Relaunch

[UninstallRun]
Filename: "{app}\{#HostExe}"; Parameters: "--unregister"; Flags: runhidden; RunOnceId: "UnregisterUser"; Check: not IsAdminInstallMode
Filename: "{app}\{#HostExe}"; Parameters: "--unregister --all-users"; Flags: runhidden; RunOnceId: "UnregisterAll"; Check: IsAdminInstallMode

[UninstallDelete]
Type: files; Name: "{app}\com.grabbit.host*.json"

[Code]
const
  RunningMutex = 'Grabbit.Running';

{ /relaunch=1 on the command line (MDM's Update now): start MDM again after installing. }
function Relaunch: Boolean;
begin
  Result := ExpandConstant('{param:relaunch|0}') = '1';
end;

{ Ask a running MDM to save and quit, and wait up to 15 s for it. }
procedure ExitRunningApp(Exe: String);
var
  Code, Waited: Integer;
begin
  if CheckForMutexes(RunningMutex) and FileExists(Exe) then
  begin
    Exec(Exe, '/exit', '', SW_HIDE, ewWaitUntilTerminated, Code);
    Waited := 0;
    while CheckForMutexes(RunningMutex) and (Waited < 15000) do
    begin
      Sleep(250);
      Waited := Waited + 250;
    end;
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  ExitRunningApp(ExpandConstant('{app}\{#AppExe}'));
  Result := '';
end;

function InitializeUninstall(): Boolean;
begin
  ExitRunningApp(ExpandConstant('{app}\{#AppExe}'));
  Result := True;
end;

{ Remove a class key only while it still points at us (another program may have taken over
  magnet links or .torrent files since). }
procedure ForgetIfOurs(Key, Ours: String);
var
  Value: String;
begin
  if RegQueryStringValue(HKA, Key + '\shell\open\command', '', Value) and (Pos(Lowercase(Ours), Lowercase(Value)) > 0) then
    RegDeleteKeyIncludingSubkeys(HKA, Key);
end;

{ This user's browser entry for the native host, when it names the user's own manifest in the
  data folder (Set it up for a copy installed for everyone, which can't write beside it): the
  uninstall for everyone (--unregister --all-users) clears the computer's entries only. }
procedure ForgetUserHost(Key: String);
var
  Value: String;
begin
  if RegQueryStringValue(HKCU, Key + '\com.grabbit.host', '', Value) and (Pos(Lowercase(ExpandConstant('{userappdata}\grabbit\')), Lowercase(Value)) = 1) then
    RegDeleteKeyIncludingSubkeys(HKCU, Key + '\com.grabbit.host');
end;

{ Asked before the files go (the sign-ins are removed by MDM itself), the list and settings
  removed after. A silent uninstall keeps both. }
var
  RemoveData: Boolean;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Value, Data: String;
  Code: Integer;
begin
  Data := ExpandConstant('{userappdata}\grabbit');
  if CurUninstallStep = usUninstall then
  begin
    ForgetIfOurs('Software\Classes\magnet', ExpandConstant('{app}\{#AppExe}'));
    if RegQueryStringValue(HKA, 'Software\Classes\.torrent', '', Value) and (CompareText(Value, 'Grabbit.torrent') = 0) then
      RegDeleteValue(HKA, 'Software\Classes\.torrent', '');
    { Default apps (upstream issue 1091): what the app registered for this user (Torrent
      settings' switch), also when this install was everyone's; only when it is this install's
      (its icon is this exe), so a portable copy's own listing stays. }
    if RegQueryStringValue(HKCU, 'Software\Grabbit\Capabilities', 'ApplicationIcon', Value) and (Pos(Lowercase(ExpandConstant('{app}\{#AppExe}')), Lowercase(Value)) > 0) then
    begin
      if RegQueryStringValue(HKCU, 'Software\RegisteredApplications', '{#AppName}', Value) and (CompareText(Value, 'Software\Grabbit\Capabilities') = 0) then
        RegDeleteValue(HKCU, 'Software\RegisteredApplications', '{#AppName}');
      RegDeleteKeyIncludingSubkeys(HKCU, 'Software\Grabbit\Capabilities');
    end;
    if RegQueryStringValue(HKCU, 'Software\Classes\Grabbit.Magnet\shell\open\command', '', Value) and (Pos(Lowercase(ExpandConstant('{app}\{#AppExe}')), Lowercase(Value)) > 0) then
      RegDeleteKeyIncludingSubkeys(HKCU, 'Software\Classes\Grabbit.Magnet');
    { Installed for everyone only: a copy installed for one user writes beside itself, and its
      own --unregister removes only what is its own, so another copy's stays. The keys of
      mdm_ipc::browsers::WINDOWS_HOST_KEYS. }
    if IsAdminInstallMode then
    begin
      ForgetUserHost('Software\Google\Chrome\NativeMessagingHosts');
      ForgetUserHost('Software\Chromium\NativeMessagingHosts');
      ForgetUserHost('Software\Microsoft\Edge\NativeMessagingHosts');
      ForgetUserHost('Software\BraveSoftware\Brave-Browser\NativeMessagingHosts');
      ForgetUserHost('Software\Mozilla\NativeMessagingHosts');
      DeleteFile(ExpandConstant('{userappdata}\grabbit\com.grabbit.host.json'));
      DeleteFile(ExpandConstant('{userappdata}\grabbit\com.grabbit.host.firefox.json'));
    end;
    RemoveData := False;
    if DirExists(Data) and not UninstallSilent() then
      RemoveData := MsgBox(CustomMessage('KeepData'), mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES;
    { The sign-ins MDM keeps in Windows Credential Manager (only MDM's own entries). }
    if RemoveData then
      Exec(ExpandConstant('{app}\{#AppExe}'), '--forget-sign-ins', '', SW_HIDE, ewWaitUntilTerminated, Code);
  end;
  if (CurUninstallStep = usPostUninstall) and RemoveData then
    DelTree(Data, True, True, True);
end;
