; Inno Setup Script for TheBestMLAWriter
; Produces a modern, professional Windows installer (.exe) with file associations and shortcuts

#define MyAppName "TheBestMLAWriter"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "TheBestMLAWriter Authors"
#define MyAppURL "https://github.com/TheBestMLAWriter"
#define MyAppExeName "the_best_mla_writer.exe"

[Setup]
AppId={{C7892341-B890-4A61-9876-1234567890AB}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
ChangesAssociations=yes
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE
OutputBaseFilename=TheBestMLAWriter-Setup-v{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "associate"; Description: "Associate .mladoc files with TheBestMLAWriter"; GroupDescription: "File Associations:"

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\sample_mla_paper.mladoc"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; Register .mladoc file extension
Root: HKA; Subkey: "Software\Classes\.mladoc"; ValueType: string; ValueName: ""; ValueData: "TheBestMLAWriter.Document"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\TheBestMLAWriter.Document"; ValueType: string; ValueName: ""; ValueData: "MLA 9th Edition Document"; Flags: uninsdeletekey; Tasks: associate
Root: HKA; Subkey: "Software\Classes\TheBestMLAWriter.Document\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: associate
Root: HKA; Subkey: "Software\Classes\TheBestMLAWriter.Document\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associate

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
