; Installateur Windows de HomeCraft Edition Vanilla (Inno Setup 6).
; Construit par Build\New-Installer.ps1 à partir du package portable (Build\package\HomeCraft-Edition-Vanilla-v<version>\).
; Définitions attendues : AppVersion, SourceDir (dossier du package), et, pour la signature, l'outil « hc » passé à ISCC (/Shc=...).
; Jamais inclus : le client World of Warcraft, Server\Data, UserData, Logs (voir New-Package.ps1).

#ifndef AppVersion
  #error "Définir AppVersion (/DAppVersion=x.y.z)"
#endif
#ifndef SourceDir
  #error "Définir SourceDir (/DSourceDir=chemin du package)"
#endif
#define AppName "HomeCraft Edition Vanilla"
#define AppExe "HomeCraft Edition Vanilla.exe"
#define Publisher "BlaMacfly"

[Setup]
AppId={{6B2F0D4C-9A7E-4F3B-8C1D-0E5A7B9C2D11}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#Publisher}
AppCopyright=© 2026 {#Publisher}. Tous droits réservés.
VersionInfoVersion={#AppVersion}
VersionInfoCompany={#Publisher}
VersionInfoDescription=Installateur {#AppName}
VersionInfoProductName={#AppName}
; Dossier inscriptible par l'utilisateur, hors Program Files (la base de données et les données extraites y sont écrites).
DefaultDirName={sd}\Jeux\{#AppName}
DisableDirPage=no
DirExistsWarning=no
UsePreviousAppDir=yes
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
LicenseFile=..\LICENCE.txt
InfoBeforeFile=package-files\AVANT-INSTALLATION.txt
OutputDir=package
OutputBaseFilename=HomeCraft-Edition-Vanilla-v{#AppVersion}-setup
SetupIconFile=..\Launcher\assets\homecraft.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
WizardStyle=modern
WizardImageFile=installer-assets\wizard-*.bmp
WizardSmallImageFile=installer-assets\wizard-small-*.bmp
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes
; Installation sans droits administrateur par défaut (dossier utilisateur) ; l'utilisateur peut choisir une installation pour tous.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
; Launcher ou services encore ouverts : l'assistant demande leur fermeture (Restart Manager).
CloseApplications=yes
RestartApplications=no
ShowLanguageDialog=no
#ifdef SignSetup
SignTool=hc
SignedUninstaller=yes
#endif

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[CustomMessages]
french.ClientPageCaption=Client World of Warcraft 1.12.1
french.ClientPageDescription=Où se trouve votre jeu ?
french.ClientPageSubCaption=HomeCraft ne fournit pas le jeu. Indiquez le dossier de votre client World of Warcraft 1.12.1 en français (il contient WoW.exe et le dossier Data). Laissez vide pour l'indiquer plus tard dans Paramètres, ou pour copier le jeu dans le sous-dossier Client de l'installation.
french.ClientPagePrompt=Dossier du client (facultatif) :
french.ClientNotFound=WoW.exe est introuvable dans ce dossier.%n%nContinuer quand même ? (Vous pourrez corriger le dossier dans Paramètres.)
french.CreateDesktopIconTask=Créer un raccourci sur le Bureau
french.ManualShortcut=Manuel d'utilisation
french.LaunchAfter=Ouvrir {#AppName} maintenant
french.UninstallKept=HomeCraft est désinstallé.%n%nVos personnages, comptes et sauvegardes (dossier UserData) et votre client (dossier Client) ont été conservés dans :%n%1%n%nSupprimez ce dossier vous-même si vous n'en avez plus besoin.

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIconTask}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Excludes: "autorun.inf"; Flags: recursesubdirs createallsubdirs ignoreversion

[Dirs]
Name: "{app}\Client"; Flags: uninsneveruninstall
Name: "{app}\UserData"; Flags: uninsneveruninstall
Name: "{app}\Logs"

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"
Name: "{group}\{cm:ManualShortcut}"; Filename: "{app}\Documentation\Manuel-Utilisateur.md"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchAfter}"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Régénérables : données extraites du client, journaux, configurations générées. UserData et Client ne sont jamais touchés.
Type: filesandordirs; Name: "{app}\Server\Data"
Type: filesandordirs; Name: "{app}\Logs"
Type: files; Name: "{app}\Server\Config\*.conf"
Type: files; Name: "{app}\Server\Config\my.ini"

[Code]
var
  ClientPage: TInputDirWizardPage;

procedure InitializeWizard;
begin
  ClientPage := CreateInputDirPage(wpSelectDir, CustomMessage('ClientPageCaption'), CustomMessage('ClientPageDescription'),
    CustomMessage('ClientPageSubCaption'), False, '');
  ClientPage.Add(CustomMessage('ClientPagePrompt'));
  { /ClientDir="..." en ligne de commande (installation silencieuse, tests) }
  ClientPage.Values[0] := ExpandConstant('{param:ClientDir|}');
end;

function NextButtonClick(CurPageID: Integer): Boolean;
var
  Dir: String;
begin
  Result := True;
  if (ClientPage <> nil) and (CurPageID = ClientPage.ID) then
  begin
    Dir := Trim(ClientPage.Values[0]);
    if (Dir <> '') and not FileExists(AddBackslash(Dir) + 'WoW.exe') then
      Result := MsgBox(CustomMessage('ClientNotFound'), mbConfirmation, MB_YESNO) = IDYES;
  end;
end;

{ Enregistre le dossier du client dans UserData\settings.json (UTF-8), seulement si aucun réglage n'existe déjà
  (mise à jour d'une installation existante : les réglages de l'utilisateur sont conservés). }
procedure CurStepChanged(CurStep: TSetupStep);
var
  Dir, Settings, Json: String;
  Lines: TArrayOfString;
begin
  if CurStep <> ssPostInstall then Exit;
  Dir := Trim(ClientPage.Values[0]);
  if Dir = '' then Exit;
  Settings := ExpandConstant('{app}\UserData\settings.json');
  if FileExists(Settings) then Exit;
  Json := Dir;
  StringChangeEx(Json, '\', '\\', True);
  StringChangeEx(Json, '"', '\"', True);
  Json := '{"client_dir":"' + Json + '"}';
  SetArrayLength(Lines, 1);
  Lines[0] := Json;
  ForceDirectories(ExpandConstant('{app}\UserData'));
  SaveStringsToUTF8File(Settings, Lines, False);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if (CurUninstallStep = usPostUninstall) and not UninstallSilent then
    MsgBox(FmtMessage(CustomMessage('UninstallKept'), [ExpandConstant('{app}')]), mbInformation, MB_OK);
end;
