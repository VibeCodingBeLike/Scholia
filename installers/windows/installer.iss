; Inno Setup Script for Scholia
; Produces a modern, professional Windows installer (.exe) with file associations and shortcuts

#define MyAppName "Scholia"
#ifndef MyAppVersion
#define MyAppVersion "0.1.0"
#endif
#define MyAppPublisher "Scholia Authors"
#define MyAppURL "https://github.com/Scholia/Scholia"
#define MyAppExeName "scholia.exe"
#ifndef MyAppExeSource
#define MyAppExeSource "..\..\target\release\" + MyAppExeName
#endif
#ifndef OutputDir
#define OutputDir "..\..\dist"
#endif
#ifndef OutputBaseFilename
#define OutputBaseFilename "Scholia-Setup-v" + MyAppVersion
#endif

[Setup]
AppId={{C7892341-B890-4A61-9876-1234567890AB}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
OutputDir={#OutputDir}
OutputBaseFilename={#OutputBaseFilename}
ChangesAssociations=yes
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "associate"; Description: "Associate .mla and .mladoc files with Scholia"; GroupDescription: "File Associations:"; Flags: checkedonce

[Files]
Source: "{#MyAppExeSource}"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion
Source: "..\..\assets\scholia-mla.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\sample_mla_paper.mla"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; Register .mla and .mladoc file extensions
Root: HKA; Subkey: "Software\Classes\.mla"; ValueType: string; ValueName: ""; ValueData: "Scholia.Document"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\.mladoc"; ValueType: string; ValueName: ""; ValueData: "Scholia.Document"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\Scholia.Document"; ValueType: string; ValueName: ""; ValueData: "MLA 9th Edition Document"; Flags: uninsdeletekey; Tasks: associate
Root: HKA; Subkey: "Software\Classes\Scholia.Document\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\scholia-mla.ico"; Tasks: associate
Root: HKA; Subkey: "Software\Classes\Scholia.Document\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associate

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
