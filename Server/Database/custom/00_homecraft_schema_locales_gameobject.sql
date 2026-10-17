-- HomeCraft Edition BC : correctif de schéma de la base monde (tbcmangos), appliqué après l'import de TBC-DB et avant
-- les compléments de traduction (ordre alphabétique des fichiers de Server/Database/custom).
--
-- Le dump TBC-DB du 2026-10-16 (tbc-all-db.zip, commit 87af76a) annonce la révision required_s2492 du core, mais sa
-- table locales_gameobject n'a pas reçu la partie « locales » de la mise à jour core s2485_01_mangos_closing_text :
-- colonnes castbarcaption_loc1..8 au lieu de opening_text_loc1..8, et closing_text_loc1..8 absentes. mangosd 15b6ddb4
-- lit ces colonnes : « query ERROR: Unknown column 'opening_text_loc1' » et noms français des objets du monde non chargés.
-- Idempotent (IF EXISTS / IF NOT EXISTS, MariaDB) : sans effet si TBC-DB corrige un jour sa table.

ALTER TABLE `locales_gameobject`
  CHANGE COLUMN IF EXISTS `castbarcaption_loc1` `opening_text_loc1` varchar(100),
  CHANGE COLUMN IF EXISTS `castbarcaption_loc2` `opening_text_loc2` varchar(100),
  CHANGE COLUMN IF EXISTS `castbarcaption_loc3` `opening_text_loc3` varchar(100),
  CHANGE COLUMN IF EXISTS `castbarcaption_loc4` `opening_text_loc4` varchar(100),
  CHANGE COLUMN IF EXISTS `castbarcaption_loc5` `opening_text_loc5` varchar(100),
  CHANGE COLUMN IF EXISTS `castbarcaption_loc6` `opening_text_loc6` varchar(100),
  CHANGE COLUMN IF EXISTS `castbarcaption_loc7` `opening_text_loc7` varchar(100),
  CHANGE COLUMN IF EXISTS `castbarcaption_loc8` `opening_text_loc8` varchar(100);

ALTER TABLE `locales_gameobject`
  ADD COLUMN IF NOT EXISTS `closing_text_loc1` varchar(100),
  ADD COLUMN IF NOT EXISTS `closing_text_loc2` varchar(100),
  ADD COLUMN IF NOT EXISTS `closing_text_loc3` varchar(100),
  ADD COLUMN IF NOT EXISTS `closing_text_loc4` varchar(100),
  ADD COLUMN IF NOT EXISTS `closing_text_loc5` varchar(100),
  ADD COLUMN IF NOT EXISTS `closing_text_loc6` varchar(100),
  ADD COLUMN IF NOT EXISTS `closing_text_loc7` varchar(100),
  ADD COLUMN IF NOT EXISTS `closing_text_loc8` varchar(100);
