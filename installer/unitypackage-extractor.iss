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
Source: "install.reg"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\アンインストール"; Filename: "{uninstallexe}"

[Run]
Filename: "regedit.exe"; Parameters: "/s ""{app}\\install.reg"""; Flags: runhidden
