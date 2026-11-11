//! Textes de l'interface en anglais : mêmes noms que `i18n::fr` (le test `memes_entrees` le vérifie).

pub const TITLE: &str = "HOMECRAFT EDITION VANILLA";
pub const WINDOW_TITLE: &str = "HomeCraft Edition Vanilla";
pub const ALREADY_RUNNING: &str = "HomeCraft Edition Vanilla is already open for this installation. Use the window that is already open (it may be minimized in the taskbar).";

// Components
pub const COMP_CLIENT: &str = "Client";
pub const COMP_DB: &str = "Database";
pub const COMP_SERVER: &str = "Server";
pub const COMP_BOTS: &str = "PlayerBots";
pub const CLIENT_OK: &str = "verified";
pub const CLIENT_REPAIR: &str = "repair needed";
pub const CLIENT_MISSING: &str = "not found";
pub const DB_STOPPED: &str = "stopped";
pub const DB_RUNNING: &str = "running";
pub const SERVER_STOPPED: &str = "stopped";
pub const SERVER_STARTING: &str = "starting";
pub const SERVER_RUNNING: &str = "running";
pub const BOTS_ON: &str = "enabled";
pub const BOTS_OFF: &str = "disabled";

// Buttons
pub const BTN_PLAY: &str = "PLAY";
pub const BTN_STOP: &str = "STOP THE SERVER";
pub const BTN_VERIFY: &str = "VERIFY FILES";
pub const BTN_REPAIR: &str = "REPAIR";
pub const BTN_BACKUP: &str = "BACK UP";
pub const BTN_RESTORE: &str = "RESTORE";
pub const BTN_SETTINGS: &str = "SETTINGS";
pub const BTN_LOGS: &str = "LOGS";
pub const BTN_START_SERVER: &str = "START THE SERVER";

// Tabs
pub const TAB_HOME: &str = "Home";
pub const TAB_ACCOUNTS: &str = "Accounts";
pub const TAB_DATABASE: &str = "Database";
pub const TAB_SETTINGS: &str = "Settings";
pub const TAB_LOGS: &str = "Logs";

// States
pub const STATE_NOT_INSTALLED: &str = "NOT INSTALLED";
pub const STATE_INSTALLED: &str = "INSTALLED";
pub const STATE_VERIFYING: &str = "VERIFYING";
pub const STATE_INITIALIZING: &str = "INITIALIZING";
pub const STATE_STARTING_DB: &str = "STARTING DATABASE";
pub const STATE_STARTING_AUTH: &str = "STARTING LOGIN SERVER";
pub const STATE_STARTING_WORLD: &str = "STARTING GAME SERVER";
pub const STATE_READY: &str = "READY";
pub const STATE_PLAYING: &str = "IN GAME";
pub const STATE_STOPPING: &str = "STOPPING";
pub const STATE_ERROR: &str = "ERROR";
pub const STATE_REPAIRING: &str = "REPAIRING";

// Progress messages
pub const MSG_IDLE: &str = "Click PLAY to start the local server and launch the game.";
pub const MSG_VERIFY: &str = "Verifying files…";
pub const MSG_INIT: &str = "Preparing the local database…";
pub const MSG_DB: &str = "Starting the database…";
pub const MSG_DB_IMPORT: &str = "First installation: importing the world data (a few minutes)…";
pub const MSG_AUTH: &str = "Starting the login server…";
pub const MSG_WORLD: &str = "Loading the world…";
pub const MSG_WORLD_FIRST_BOTS: &str = "First start: preparing the bots' equipment (only once, 10 to 30 minutes depending on the PC)…";
pub const MSG_READY: &str = "Server running. You can play.";
pub const MSG_PLAYING: &str = "The game is running. The server stays up as long as this window is open.";
pub const MSG_STOPPING: &str = "Stopping the services cleanly…";
pub const MSG_STOPPED: &str = "Services stopped.";
pub const MSG_CLOSING: &str = "Closing: stopping the services…";
pub const MSG_NOT_INSTALLED: &str = "Some components are missing: use REPAIR or reinstall.";

// Accounts
pub const ACC_TITLE: &str = "Game accounts";
pub const ACC_USER: &str = "Username";
pub const ACC_PASS: &str = "Password";
pub const ACC_GM: &str = "GM level";
pub const ACC_LAST: &str = "Last login";
pub const ACC_CREATE: &str = "Create the account";
pub const ACC_REFRESH: &str = "Refresh";
pub const ACC_NEED_SERVER: &str = "The game server must be running to create an account (offline creation: coming later).";
pub const ACC_HINT: &str = "GM levels: 0 player, 1 moderator, 2 game master, 3 administrator. From 2 upwards, GM commands (.go, .additem, .levelup…) are available and every bot obeys your orders.";
pub const ACC_BOT_PREFIX: &str = "Names starting with RNDBOT are reserved for the random bots' accounts.";
pub const ACC_SET_PASS: &str = "Change the password";
pub const ACC_SET_GM: &str = "Apply the GM level";
pub const ACC_DELETE: &str = "Delete";
pub const ACC_DELETE_CONFIRM_TITLE: &str = "Delete this account permanently?";
pub const ACC_DELETE_CONFIRM: &str = "The account and ALL its characters (items, mail, progress) will be deleted permanently. This cannot be undone: make a backup first (Backups tab) if in doubt.";
pub fn acc_delete_online(user: &str) -> String {
    format!("Account {} has a character in game: log it out (back to the character screen or close the game), then try again.", user.to_ascii_uppercase())
}
pub const ACC_DELETED: &str = "Account deleted:";
pub const ACC_DELETE_FAIL: &str = "Deletion impossible";
pub const ACC_DELETE_SELF_WARN: &str = "Warning: this is the account that logged in most recently.";
pub const ACC_CREATE_FAIL: &str = "Account creation impossible: the server did not record it (see the Logs tab).";
pub const ACC_INVALID: &str = "Username: 1 to 16 letters or digits. Password: 4 to 16 characters, no spaces.";

// Database
pub const DB_TITLE: &str = "Database explorer";
pub const DB_OPEN_HEIDI: &str = "Open in HeidiSQL";
pub const DB_HEIDI_MISSING: &str = "HeidiSQL is not installed in Tools/HeidiSQL.";
pub const DB_NEED_DB: &str = "The database must be running.";
pub const DB_BASE: &str = "Database";
pub const DB_TABLE: &str = "Table";
pub const DB_LOAD: &str = "Show (first 200 rows)";
pub const DB_SQL: &str = "SQL query (read only: SELECT / SHOW / DESCRIBE)";
pub const DB_RUN: &str = "Run";
pub const DB_ROWS: &str = "rows";
pub const DB_READONLY_ONLY: &str = "This statement is not a read: use \"Run (write)\" after checking it.";
pub const DB_INTRO: &str = "Built-in explorer: pick a table in the tree; double-click a cell to edit it (Enter confirms, Esc cancels), insert or delete rows. Tables without a primary key are read only. Every write is logged.";
pub const DB_TAB_DATA: &str = "Data";
pub const DB_TAB_STRUCTURE: &str = "Structure";
pub const DB_TAB_SQL: &str = "SQL query";
pub const DB_FILTER: &str = "Filter (WHERE clause)";
pub const DB_APPLY_FILTER: &str = "Apply";
pub const DB_PREV: &str = "◀ Previous";
pub const DB_NEXT: &str = "Next ▶";
pub const DB_INSERT: &str = "Insert a row";
pub const DB_DELETE_ROW: &str = "Delete the row";
pub const DB_DELETE_ROW_CONFIRM_TITLE: &str = "Delete this row?";
pub const DB_DELETE_ROW_CONFIRM: &str = "The row will be deleted permanently from the table.";
pub const DB_NO_PK: &str = "Table without a primary key: editing disabled in the grid (use the SQL query tab or HeidiSQL).";
pub const DB_RUN_READ: &str = "Run (read)";
pub const DB_RUN_WRITE: &str = "Run (write)";
pub const DB_WRITE_CONFIRM_TITLE: &str = "Run this write statement?";
pub const DB_WRITE_CONFIRM: &str = "It changes the database permanently. Make a backup if in doubt.";
pub const DB_AFFECTED: &str = "row(s) affected";
pub const DB_HISTORY: &str = "History";
pub const DB_COL_NAME: &str = "Column";
pub const DB_COL_TYPE: &str = "Type";
pub const DB_COL_NULL: &str = "NULL";
pub const DB_COL_KEY: &str = "Key";
pub const DB_COL_DEFAULT: &str = "Default";
pub const DB_INSERT_TITLE: &str = "New row";
pub const DB_INSERT_HINT: &str = "Leave empty to use the default value; type NULL for a null value.";
pub const DB_TOTAL: &str = "rows in total";

// Settings
pub const SET_CLIENT_DIR: &str = "WoW 1.12.1 client folder";
pub const SET_CLIENT_HINT: &str = "Empty = the Client/ folder of the installation. Two clients are recognized: the original client (WoW.exe 1.12.1, 32-bit) and Benilla (free client rewritten in Rust, 64-bit). The client is never downloaded: provide your own.";
pub const SET_CLIENT_DETECTED: &str = "Client detected:";
pub const SET_CLIENT_MISSING_MPQ: &str = "archives missing in Data:";
pub const SET_CLIENT_BENILLA_HINT: &str = r"Benilla: the launcher sets benilla-config\config.toml (realm 127.0.0.1, windowed or borderless full screen, window size); AddOns go into benilla-config\AddOns. VanillaFixes and DXVK are not used (native 64-bit rendering). The game is in French if the Data MPQs come from a frFR client. Do not put encrypted patches there (patch-3/4/5 of some repacks): Benilla cannot read them. If base.MPQ is missing in Data, the launcher creates an empty replacement archive there for the server extractors (Benilla ignores it).";
pub const SET_CLIENT_SCAN: &str = "Find clients";
pub const SET_CLIENT_SCAN_HINT: &str = r"Looks for a valid client in Client\ and in the folders next to the installation (for example on the Desktop).";
pub const SET_CLIENT_USE: &str = "Use";
pub const SET_CLIENT_IN_USE: &str = "In use";
pub const SET_VF_BENILLA: &str = "Not applicable with Benilla: it has its own 64-bit renderer (Vulkan / Direct3D 12).";
pub const SPAWN_BLOCKED_APP_CONTROL: &str = "Windows blocked this program: it is usually Windows 11's \"Smart App Control\", which refuses programs not signed by a recognized publisher (the CMaNGOS server tools are not). Open Windows Security > App & browser control > Smart App Control settings, and choose Off; see the manual, Troubleshooting section.";
pub const SPAWN_BLOCKED_ANTIVIRUS: &str = "The antivirus blocked this program: add the HomeCraft folder to the antivirus exclusions, restore the file from quarantine, then use VERIFY FILES.";
pub const EXTRACT_MISSING: &str = "Server data extraction impossible: missing";
pub const EXTRACT_MISSING_HINT: &str = "The server extractors need these archives of the original client (Benilla does not). Copy them from your 1.12.1 client into this Data folder, then click PLAY or REPAIR: they do not bother Benilla.";
pub const CLIENT_KIND_ORIGINAL: &str = "Original client (WoW.exe)";
pub const CLIENT_KIND_BENILLA: &str = "Benilla";
pub const CLIENT_KIND_NONE: &str = "no client recognized";
pub const SET_WINDOWED: &str = "Windowed game (recommended: change the mode here rather than in the game options)";
pub const SET_WINDOWED_HINT: &str = "The launcher also caps the resolution to your screen: a 1.12.1 client windowed at 1920x1080 on a 1920x1080 screen shows a black screen.";
pub const BTN_KILL_GAME: &str = "CLOSE THE GAME";
pub const BTN_KILL_GAME_HINT: &str = "Forces WoW.exe to close (black screen, frozen game). Progress not saved by the server since the last automatic save is lost.";
pub const SET_AHBOT: &str = "Stocked auction house (AHBot)";
pub const SET_AHBOT_HINT: &str = "The server lists and buys items in the three auction houses (Alliance, Horde and goblin).";
pub const SET_BOTS: &str = "PlayerBots companions (.partybot commands)";
pub const SET_RATE_XP: &str = "Experience multiplier";
pub const SET_SAVE: &str = "Save";
pub const SET_SAVED: &str = "Settings saved.";

// Logs / diagnostics
pub const LOG_TITLE: &str = "Launcher log and server console";
pub const LOG_CONSOLE: &str = "mangosd console command";
pub const LOG_SEND: &str = "Send";
pub const LOG_DIAG: &str = "Diagnostics";

pub const NOT_YET: &str = "Coming later";

// Backups
pub const TAB_BACKUPS: &str = "Backups";
pub const BK_TITLE: &str = "Backups of characters and accounts";
pub const BK_HINT: &str = "Each backup holds the characters and realmd databases (consistent export, locked tables), your settings and a manifest of checksums. Folder: UserData/Sauvegardes, kept across updates.";
pub const BK_NOW: &str = "Back up now";
pub const BK_RESTORE_SEL: &str = "Restore the selection";
pub const BK_DELETE_SEL: &str = "Delete the selection";
pub const BK_OPEN_DIR: &str = "Open the folder";
pub const BK_DIAG: &str = "Export diagnostics";
pub const BK_DATE: &str = "Date";
pub const BK_LABEL: &str = "Label";
pub const BK_CHARS: &str = "Characters";
pub const BK_ACCOUNTS: &str = "Accounts";
pub const BK_SIZE: &str = "Size";
pub const BK_NONE: &str = "No backup yet.";
pub const BK_PLAYING_WARN: &str = "The game is running: progress not yet saved by the server is forced first (saveall), then exported.";
pub const BK_RESTORE_CONFIRM_TITLE: &str = "Confirm the restore";
pub const BK_RESTORE_CONFIRM: &str = "The current characters and accounts will be REPLACED by those of this backup. A safety backup of the current state is made first. The game server will be stopped.";
pub const BK_DELETE_CONFIRM_TITLE: &str = "Confirm the deletion";
pub const BK_DELETE_CONFIRM: &str = "This backup will be deleted permanently from the disk.";
pub const BK_RESTORE_PLAYING: &str = "Cannot restore while the game is running: close the game first.";
pub const CONFIRM: &str = "Confirm";
pub const CANCEL: &str = "Cancel";
pub const CLOSE: &str = "Close";

// Messages
pub const MSG_BACKUP: &str = "Backing up…";
pub const MSG_BACKUP_DONE: &str = "Backup finished.";
pub const MSG_RESTORE: &str = "Restoring…";
pub const MSG_RESTORE_DONE: &str = "Restore finished. You can restart the server.";
pub const MSG_VERIFYING_FILES: &str = "Verifying managed files…";
pub const MSG_REPAIR: &str = "Repairing…";
pub const MSG_REPAIR_DONE: &str = "Repair finished: see the report.";
pub const MSG_EXTRACT: &str = "Extracting the client data…";
pub const MSG_MMAPS: &str = "Generating mmaps (several tens of minutes to a few hours)…";
pub const MSG_DIAG_DONE: &str = "Diagnostics exported: ";

// Verification report
pub const RPT_TITLE: &str = "Verification report";
pub const RPT_MANIFEST_MISSING: &str = "Manifest Build/files-manifest.json missing: only the presence of the components was checked.";
pub const RPT_FILES_OK: &str = "Managed files: all present and correct";
pub const RPT_MISSING: &str = "Missing files";
pub const RPT_ALTERED: &str = "Altered files";
pub const RPT_DATA: &str = "Data extracted from the client";
pub const RPT_DB: &str = "Databases";
pub const RPT_DB_OK: &str = "four databases present, complete schema";
pub const RPT_DB_UNKNOWN: &str = "not checked (database stopped)";
pub const RPT_UNREPAIRABLE: &str = "Missing or altered files that REPAIR cannot rebuild: reinstall the package (your characters and backups are kept).";
pub const RPT_REPAIR_HINT: &str = "REPAIR regenerates the configurations, reimports missing databases (never the characters) and re-extracts the client data if needed.";
pub const BTN_MMAPS: &str = "Generate mmaps (long)";

// PlayerBots
pub const TAB_BOTS: &str = "PlayerBots";
pub const BOT_TITLE: &str = "PlayerBots (cmangos/playerbots module)";
pub const BOT_INTRO: &str = "The world is populated by bots that play like players: they level up, do their quests, hunt, go to dungeons and battlegrounds, form groups and guilds. Invite one into your group (right-click its portrait, or /invite Name): it follows you, fights with you and obeys the orders below. With the server running, saved settings apply at once; the number of bot accounts only changes at the next start.";
pub const BOT_ENABLE: &str = "Enable PlayerBots";
pub const BOT_SAVE: &str = "Save the PlayerBots settings";
pub const BOT_POOL: &str = "Random bot characters (RNDBOT accounts, created by the server)";
pub const BOT_POOL_NONE: &str = "none yet: they are created at the next server start (a few minutes the first time).";
pub const BOT_DELETE: &str = "Delete all random bots";
pub const BOT_DELETE_HINT: &str = "Server running: it is stopped during the operation then restarted automatically. On restart, new bots are created (level 1 or random depending on the settings), unless \"Log in the random bots\" is unchecked.";
pub const BOT_DELETE_CONFIRM_TITLE: &str = "Delete the random bots";
pub const BOT_DELETE_CONFIRM: &str = "All RNDBOT accounts and their characters will be deleted permanently (items, bot guilds and progress included). Your own accounts and characters are not affected.";
pub const BOT_DELETE_FAIL: &str = "Deletion impossible";
pub const BOT_CMDS: &str = "Commands to type in the game chat";
pub const BOT_ORDERS: &str = "Orders for your bots (in the party channel /p, or whispering a bot)";

// World
pub const TAB_WORLD: &str = "World";
pub const WORLD_TITLE: &str = "World settings (mangosd.conf)";
pub const WORLD_INTRO: &str = "The most useful game server settings, with their Vanilla default values. With the server running, saved changes apply at once (the server rereads its configuration); some, like world content by patch, only take effect after a restart. The other keys of the file stay managed automatically by the launcher.";
pub const WORLD_SAVE: &str = "Save the world settings";
pub const OPT_DEFAULT: &str = "Default";
pub const OPT_RESET_ALL: &str = "Reset everything to Vanilla values";
pub const OPT_MODIFIED: &str = "modified";

// Client patches
pub const TAB_PATCHES: &str = "Patches";
pub const PATCH_TITLE: &str = "Client patches (extra MPQ archives)";
pub const PATCH_INTRO: &str = "After its official archives, the client loads the patch-X.MPQ files of Client/Data (X: a digit from 3 to 9 or a letter from A to Z; later ones take priority over earlier ones). You bring your own patches (HD graphics, interface, sounds): the launcher neither ships nor downloads any. Disabling renames the archive to .off without deleting it.";
pub const PATCH_SERVER_NOTE: &str = "Important: the server extraction tools only read the official archives. A patch therefore has no effect on the server: purely visual or sound changes work, but a patch that adds races, items or spells on the client side will not be recognized by the server.";
pub const PATCH_NAME: &str = "Archive";
pub const PATCH_SIZE: &str = "Size";
pub const PATCH_STATE: &str = "State";
pub const PATCH_OFFICIAL: &str = "official";
pub const PATCH_ACTIVE: &str = "active";
pub const PATCH_INACTIVE: &str = "disabled";
pub const PATCH_NOT_LOADED: &str = "name not loaded by the client";
pub const PATCH_ENABLE: &str = "Enable";
pub const PATCH_DISABLE: &str = "Disable";
pub const PATCH_REMOVE: &str = "Delete";
pub const PATCH_ADD: &str = "Add a patch…";
pub const PATCH_OPEN_DIR: &str = "Open the Data folder";
pub const PATCH_PLAYING: &str = "The game is running: close it before changing patches (files in use).";
pub const PATCH_ADDED: &str = "Patch added:";
pub const PATCH_ADD_FAIL: &str = "Adding impossible";
pub const PATCH_REMOVE_CONFIRM_TITLE: &str = "Delete the patch";
pub const PATCH_REMOVE_CONFIRM: &str = "The archive will be deleted permanently from Client/Data. Prefer \"Disable\" to keep it.";
pub const PATCH_NONE: &str = "No extra patch: only the official archives are present.";
pub const PATCH_BUILD: &str = "Create a patch from a folder…";
pub const PATCH_BUILD_HINT: &str = "Turns a file tree (e.g. an unzipped VanillaHD \"Patch-A-main\" repository) into a patch-X.MPQ archive in the 1.12.1 client format. Several minutes for a big patch.";
pub const PATCH_BUILDING: &str = "Building the MPQ archive…";
pub const PATCH_BUILT: &str = "Patch created:";
pub const PATCH_LOOSE_NOTE: &str = "Loose files placed in Data (e.g. Data/Spell.dbc) are also read by the client, with priority over archives; they are not managed here.";

// AddOns
pub const ADDON_TITLE: &str = "Client AddOns (Interface\\AddOns, or benilla-config\\AddOns with Benilla)";
pub const ADDON_INTRO: &str = "Install your 1.12.1 AddOns (Questie, pfUI, Bagnon…) from their zip archive: the launcher finds the AddOn folder(s) (Name.toc file) and puts them in the right place, even if the archive comes from GitHub (\"Name-master\" folder). Disabling renames the folder to .off; final per-character activation is done in the game, AddOns button of the character screen. No AddOn is shipped or downloaded.";
pub const ADDON_ADD: &str = "Add an AddOn (zip)…";
pub const ADDON_OPEN_DIR: &str = "Open the AddOns folder";
pub const ADDON_NAME: &str = "Folder";
pub const ADDON_TITLE_COL: &str = "Title";
pub const ADDON_VERSION: &str = "Version";
pub const ADDON_INTERFACE: &str = "Interface";
pub const ADDON_STATE: &str = "State";
pub const ADDON_ACTIVE: &str = "active";
pub const ADDON_INACTIVE: &str = "disabled";
pub const ADDON_NO_TOC: &str = "no .toc (ignored by the game)";
pub const ADDON_OLD_INTERFACE: &str = "Interface ≠ 11200: made for another version, the game may flag it \"out of date\" (tick \"Load out of date AddOns\").";
pub const ADDON_PLAYING: &str = "The game is running: close it before changing AddOns.";
pub const ADDON_ADDED: &str = "AddOn installed:";
pub const ADDON_ADD_FAIL: &str = "Installation impossible";
pub const ADDON_REMOVE: &str = "Delete";
pub const ADDON_REMOVE_CONFIRM_TITLE: &str = "Delete the AddOn";
pub const ADDON_REMOVE_CONFIRM: &str = "The AddOn folder will be deleted permanently (its settings in WTF are kept).";
pub const ADDON_NONE: &str = "No AddOn installed.";

// Navigation maps (mmaps)
pub const HOME_MMAPS: &str = "Navigation maps (mmaps)";
pub const HOME_MMAPS_PRESENT: &str = "present";
pub const HOME_MMAPS_ABSENT: &str = "missing: generated at the next server start";
pub const HOME_MMAPS_RUNNING: &str = "generating for";
pub const HOME_MMAPS_DONE_RESTART: &str = "generated: active at the next server start";
pub const HOME_MMAPS_FAILED: &str = "generation failed:";
pub const MMAPS_ALREADY_RUNNING: &str = "Mmaps generation is already running.";
pub const BTN_MMAPS_HINT: &str = "Regenerates the navigation maps in the background (15 to 30 minutes), used at the next server start.";

// Automatic backup
pub const SET_AUTO_BACKUP: &str = "Back up the characters automatically each time the game closes";
pub const SET_AUTO_BACKUP_KEEP: &str = "Automatic backups kept";
pub const SET_AUTO_BACKUP_HINT: &str = "Manual and safety backups are never deleted by this rotation.";
/// Libellé enregistré dans la sauvegarde et reconnu par la rotation : identique dans les deux langues.
pub const BK_AUTO_LABEL: &str = super::fr::BK_AUTO_LABEL;

// World database reimport
pub const BTN_WORLD_REIMPORT: &str = "Reimport the world database…";
pub const WORLD_REIMPORT_HINT: &str = "Recreates the mangos database (quests, creatures, items, translations) from the package files, without touching characters or accounts. Useful after a server update or to undo changes made in the database explorer.";
pub const WORLD_REIMPORT_CONFIRM_TITLE: &str = "Reimport the world database";
pub const WORLD_REIMPORT_CONFIRM: &str = "The world database (mangos) will be deleted then reimported from the package, HomeCraft additions included. Characters, accounts and backups are kept. Any change made in the world database will be lost. The game server will be stopped; it takes one to two minutes.";
pub const WORLD_REIMPORT_PLAYING: &str = "Cannot reimport the world database while the game is running: close the game first.";
pub const MSG_WORLD_REIMPORT: &str = "Reimporting the world database…";
pub const MSG_WORLD_REIMPORT_DONE: &str = "World database reimported. You can restart the server.";

// VanillaFixes / DXVK
pub const SET_VF: &str = "Launch the game through VanillaFixes (fixes stutter and animation timing)";
pub const SET_VF_HINT: &str = "Copies VanillaFixes.exe, VfPatcher.dll and dlls.txt into the client folder and starts the game through this launcher (DLL injected at startup, unloaded at the login screen). Some antiviruses wrongly flag this kind of launcher. MIT licence.";
pub const SET_DXVK: &str = "DXVK rendering: translate Direct3D 9 to Vulkan (recent graphics cards)";
pub const SET_DXVK_HINT: &str = "Copies d3d9.dll (DXVK 2.6.1, zlib licence) next to WoW.exe. Worth trying if the display misbehaves on Windows 11; needs a Vulkan driver. Unticking removes the file.";
pub const SET_VF_MISSING: &str = "VanillaFixes is not present in Tools/VanillaFixes.";

// Read-only media / portable installation
pub const RO_TITLE: &str = "This folder is read only (CD, DVD or protected folder).";
pub const RO_TEXT: &str = "The game needs to write its database and the data extracted from your client. Choose a folder on your hard disk: all the content will be copied there, then the launcher will open from that folder.";
pub const RO_INSTALL: &str = "Install on this PC…";
pub const RO_PICK_TITLE: &str = "Choose the installation folder";
pub const MSG_INSTALLING: &str = "Copying files to the disk…";
pub const MSG_INSTALLED: &str = "Copy finished: opening the installed launcher.";
pub const SET_BROWSE: &str = "Browse…";
pub const MSG_CLIENT_NEEDED: &str = "The WoW 1.12.1 client cannot be found or is invalid: put it in the Client folder of the installation (it is never shipped by this package).";
pub const MSG_FIRST_EXTRACT: &str = "First installation: extracting the data of your client (a few minutes)…";
pub const MSG_FIRST_MMAPS: &str = "First installation: generating the navigation maps, only once (15 to 30 minutes):";

pub fn bot_load(max: u32, min: u32) -> String {
    format!("Up to {max} bots online (at least {min}): each bot has a full AI, the server will need a lot of memory and CPU. Lower this number if the game or the PC slows down.")
}

pub fn mmaps_first_install(percent: u32, remaining_secs: Option<u64>) -> String {
    let rest = match remaining_secs {
        None => "estimating the remaining time…".to_string(),
        Some(s) if s < 60 => "almost done".to_string(),
        Some(s) => format!("about {} min left", (s + 59) / 60),
    };
    format!("{MSG_FIRST_MMAPS} {percent}%, {rest}")
}

pub fn mmaps_running(elapsed_min: u64, remaining_secs: Option<u64>) -> String {
    let rest = match remaining_secs {
        None => "remaining time: estimating…".to_string(),
        Some(s) if s < 60 => "almost done".to_string(),
        Some(s) if s < 3600 => format!("about {} min left", (s + 59) / 60),
        Some(s) => format!("about {} h {:02} left", s / 3600, (s % 3600) / 60),
    };
    format!("{HOME_MMAPS_RUNNING} {elapsed_min} min, {rest} (the game stays playable)")
}

pub fn botgen_players_in_world(n: u64) -> String {
    format!("{n} of your characters {} in game: log out (back to the character screen), then try again. The server must be stopped for a few seconds during the operation.", if n > 1 { "are" } else { "is" })
}

pub fn acc_exists(user: &str) -> String {
    format!("Account {} already exists.", user.to_ascii_uppercase())
}

pub fn acc_created(user: &str, gm: u8) -> String {
    format!("Account {} created (GM level {gm}).", user.to_ascii_uppercase())
}

pub fn profile_already_running(name: &str) -> String {
    format!("Profile \"{name}\" already has a client open.")
}
pub const NET_NOT_GUEST: &str = "Join a game first (Multiplayer tab).";
pub const NET_GUEST_NO_SERVER: &str = "You joined a host's game: your local server cannot start until you leave that game (Multiplayer tab).";
