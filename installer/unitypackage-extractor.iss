; unity-package インストーラー Inno Setup スクリプト
[Setup]
AppName=unitypackage-extractor
AppVersion=0.0.4
DefaultDirName={pf}\unitypackage-extractor
DefaultGroupName=unitypackage-extractor
OutputBaseFilename=unitypackage-extractor-installer
Compression=lzma
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"

[Files]
Source: "..\target\x86_64-pc-windows-msvc\release\unitypackage-extractor.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\アンインストール"; Filename: "{uninstallexe}"

[Registry]
Root: HKCR; Subkey: "SystemFileAssociations\.unitypackage\shell\unitypackage-extractor"; ValueType: string; ValueName: ""; ValueData: "unitypackage-extractorで展開"; Flags: uninsdeletekey
Root: HKCR; Subkey: "SystemFileAssociations\.unitypackage\shell\unitypackage-extractor\command"; ValueType: string; ValueName: ""; ValueData: '"{app}\unitypackage-extractor.exe" "%1"'; Flags: uninsdeletekey
