/*
 * HomeCraft Edition Vanilla — couche de compatibilité libmySQL.dll pour VMaNGOS.
 *
 * Le libmySQL.dll livré avec VMaNGOS (client MySQL 5.5.62) dépend du runtime Visual C++ 2008 (Microsoft.VC90.CRT),
 * absent d'un Windows 10/11 récent : mangosd et realmd ne démarrent pas sans lui. Cette bibliothèque le remplace :
 * toutes les fonctions mysql_* sont redirigées (redirections d'exportation du fichier .def, générées à partir des
 * exportations de libmariadb.dll) vers le client MariaDB Connector/C livré avec la base (libmariadb.dll), compatible
 * binairement avec l'API cliente MySQL 5.5 pour ce qu'utilise VMaNGOS (MYSQL_FIELD, MYSQL_BIND, mysql_options).
 * Seules les trois fonctions de threads, que Connector/C n'exporte plus (elles n'y ont plus d'effet), sont écrites ici.
 *
 * Compilé sans bibliothèque C (/Zl, /NOENTRY) : aucune dépendance hors libmariadb.dll.
 */

/* my_bool mysql_thread_init(void) : 0 = succès. Connector/C initialise ses données de thread lui-même. */
__declspec(dllexport) char mysql_thread_init(void) { return 0; }

/* void mysql_thread_end(void) : rien à libérer. */
__declspec(dllexport) void mysql_thread_end(void) { }

/* unsigned int mysql_thread_safe(void) : Connector/C est toujours compilé pour les threads. */
__declspec(dllexport) unsigned int mysql_thread_safe(void) { return 1; }
