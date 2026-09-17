# Smart Library

Το Smart Library είναι desktop εφαρμογή για **Windows και Linux** που καταγράφει και ταξινομεί έγγραφα χωρίς να αλλάζει τα πρωτότυπα. Η εφαρμογή παραμένει σχεδιασμένη ως **read-only Safe Mode** για τα κανονικά folder scans.

## Τι λειτουργεί στην έκδοση 0.3.4

- επιλογή φακέλου με το native παράθυρο του λειτουργικού,
- αναδρομική σάρωση έως 50.000 κατάλληλων αρχείων,
- αυτόματος αποκλεισμός system folders των Windows/Linux, Steam/Epic/Xbox libraries, `Program Files`, caches και executables,
- τοπικό SQLite index με source path και ιστορικό scans,
- SHA-256 για αναγνώριση διπλοτύπων (έως 64 MB ανά αρχείο),
- αρχική ταξινόμηση για σκάφη/εργασία, οικονομικά, ακίνητα, οχήματα, νομικά, manuals και φωτογραφίες,
- confidence score και φάκελος `_Needs_Review`,
- πραγματικό preview για PDF, εικόνες και text/CSV/JSON/XML έως τα ασφαλή όρια μεγέθους,
- τοπική εξαγωγή κειμένου από ψηφιακά PDF και απλά αρχεία κειμένου,
- τοπική εξαγωγή κειμένου από σύγχρονα Word (`.docx`), Excel (`.xlsx`) και PowerPoint (`.pptx`), χωρίς να απαιτείται Microsoft Office,
- αποθήκευση του εξαγόμενου κειμένου στο SQLite FTS5 index,
- ταξινόμηση από το περιεχόμενο, με ειδική αναγνώριση για CV και παραστατικά,
- σαφής ένδειξη για σαρωμένα PDF που χρειάζονται OCR,
- αναζήτηση στο πλήρες εξαγόμενο κείμενο,
- έγκριση ή παράβλεψη μιας πρότασης, μόνο ως εγγραφή στο index,
- άνοιγμα ενός επιλεγμένου αρχείου με την προεπιλεγμένη εφαρμογή του λειτουργικού,
- εμφάνιση και επιλογή του αρχείου μέσα στον φάκελό του στον Windows Explorer ή στον Linux file manager,
- προεπισκόπηση CSV ως κανονικός πίνακας, με αυτόματη αναγνώριση κόμματος, ελληνικού ερωτηματικού/semicolon, tab ή pipe,
- ανάγνωση UTF-8, UTF-16 και ελληνικών CSV παλαιότερης κωδικοποίησης Windows-1253,
- ξεχωριστή κατηγορία `Σχέδια & CAD` για DWG, DXF, DWF, DWT, STEP/STP, IGES/IGS, STL, 3MF και OBJ,
- ξεχωριστή κατηγορία `Ήχος & Βίντεο` για MP3, WAV, FLAC, M4A, AAC, OGG, OPUS, WMA, MP4, M4V, MOV, MKV, AVI και WebM,
- εσωτερικός audio player για συμβατά αρχεία έως 32 MB και video player έως 40 MB,
- indexing για αρχεία ναυσιπλοΐας GPX, KML, KMZ και NMEA, με ανάγνωση κειμένου για GPX/KML/NMEA,
- indexing για RAW φωτογραφίες DNG/CR2/CR3/NEF/ARW/ORF/RW2 και e-books EPUB/MOBI,
- ρυθμίσεις φίλτρων ανά ομάδα και ανά επέκταση αρχείου, με τοπική αποθήκευση στη SQLite,
- σύνδεση Gmail μέσω κρυπτογραφημένου IMAP και Google App Password, χωρίς Google Cloud project ή billing,
- χρήση μόνο read-only εντολών IMAP (`EXAMINE`, `SEARCH`, `BODY.PEEK`) — χωρίς αποστολή, μετακίνηση, επισήμανση ή διαγραφή email,
- αποθήκευση του App Password στο Windows Credential Manager ή στο Linux Secret Service/keyring, ποτέ στη SQLite ή σε αρχείο ρυθμίσεων,
- ελεγχόμενη λήψη έως 50/100/250/500 email ανά batch με κανονική Gmail search query,
- λήψη μόνο συνημμένων των ενεργών τύπων στο `Documents/Smart Library/Email Inbox/Gmail`,
- SHA-256 πριν από κάθε εγγραφή, αποφυγή δεύτερου φυσικού αντιγράφου και πλήρες Gmail provenance,
- εμφάνιση λογαριασμού, αποστολέα, θέματος, ημερομηνίας και αρχικού ονόματος στην προεπισκόπηση,
- καμία εντολή διαγραφής, μετακίνησης, μετονομασίας ή αντιγραφής πρωτοτύπου.

### Interface και προεπισκόπηση

Ο χειρισμός επιλογής αρχείου γίνεται κεντρικά στο table body, ώστε να παραμένει
ενεργός μετά από scan ή ανανέωση της λίστας. Υποστηρίζονται επίσης `Enter` και
`Space` σε επιλεγμένη γραμμή. Το τεχνικό Windows prefix `\\?\` αφαιρείται μόνο
από την οπτική παρουσίαση των paths.

Με κάθε νέα επιλογή αρχείου, το δεξί panel επιστρέφει αυτόματα στην κορυφή. Κάτω
από την ταξινόμηση εμφανίζει αν το κείμενο διαβάστηκε, αν χρειάζεται OCR ή αν ο
συγκεκριμένος τύπος αρχείου δεν υποστηρίζεται ακόμη.

Στην έκδοση 0.2.2 διορθώθηκε η εναλλαγή του δεξιού panel: το μήνυμα «Επίλεξε ένα
αρχείο» αφαιρείται πλήρως μόλις γίνει επιλογή και η προεπισκόπηση εμφανίζεται
ακριβώς στην ίδια θέση, χωρίς να απαιτείται κύλιση.

Στην έκδοση 0.2.3 υπάρχουν κάτω από το path τα κουμπιά «Άνοιγμα αρχείου» και
«Στον φάκελο». Το πρώτο χρησιμοποιεί την προεπιλεγμένη εφαρμογή των Windows και
το δεύτερο ανοίγει τον Explorer με επιλεγμένο το πραγματικό αρχείο.

Στην έκδοση 0.2.4 τα τεχνικά Windows paths τύπου `\\?\C:\...` μετατρέπονται στη
μορφή που καταλαβαίνει το Windows Shell πριν από το άνοιγμα. Για CSV και απλά
αρχεία κειμένου γίνεται δεύτερη προσπάθεια με το Σημειωματάριο αν δεν υπάρχει
σωστή προεπιλεγμένη εφαρμογή. Το CSV εμφανίζεται επίσης ως πίνακας στην εσωτερική
προεπισκόπηση.

Στην έκδοση 0.2.5 προστέθηκαν οι κατηγορίες «Σχέδια & CAD» και «Ήχος & Βίντεο».
Τα συνηθισμένα audio formats αναπαράγονται μέσα στο δεξί panel. Μικρά MP4/MOV/M4V/WebM
προβάλλονται επίσης εσωτερικά. Τα DWG και τα υπόλοιπα CAD formats καταγράφονται,
ταξινομούνται, αναζητούνται, ανοίγουν με το εγκατεστημένο CAD πρόγραμμα και
εμφανίζονται στον Explorer. Η πραγματική σχεδίαση DWG μέσα στην εφαρμογή δεν
περιλαμβάνεται ακόμη, επειδή απαιτεί ξεχωριστό CAD renderer.

Στην έκδοση 0.2.6 το γρανάζι ανοίγει τις «Ρυθμίσεις σάρωσης». Εκεί μπορούν να
ενεργοποιηθούν ή να απενεργοποιηθούν ολόκληρες ομάδες — έγγραφα, εικόνες,
CAD/3D, ήχος, βίντεο, ναυσιπλοΐα, συμπιεσμένα αρχεία και email — καθώς και κάθε
επέκταση ξεχωριστά. Η επιλογή αποθηκεύεται τοπικά και εφαρμόζεται στην επόμενη
σάρωση του φακέλου. Αν ένας τύπος απενεργοποιηθεί, αφαιρείται μόνο η εγγραφή του
από το index κατά την επόμενη σάρωση· το πραγματικό αρχείο δεν αλλάζει.

Στην έκδοση 0.3.0 οι ρυθμίσεις περιλαμβάνουν και σύνδεση Gmail. Η εξουσιοδότηση
ανοίγει στον κανονικό browser και ζητά μόνο `gmail.readonly`. Ο χρήστης επιλέγει
το Gmail query και το ανώτατο πλήθος email κάθε batch. Τα συνημμένα ελέγχονται
πρώτα ως προς τύπο, μέγεθος και SHA-256 και μετά γράφονται σε ελεγχόμενο inbox.
Αν υπάρχει ήδη το ίδιο hash, αποθηκεύεται μόνο η νέα εγγραφή προέλευσης.

Στην έκδοση 0.3.1 διορθώθηκε η σύγχυση της λέξης `Audio` με τη μάρκα `Audi`.
Τα CAD, audio, video και αρχεία πλοήγησης ταξινομούνται πλέον πρώτα από την
επέκτασή τους. Το script επαλήθευσης σταματά επίσης σωστά αν αποτύχει οποιοδήποτε
`cargo` check ή unit test.

Στην έκδοση 0.3.2 η νέα σύνδεση Gmail χρησιμοποιεί IMAP μέσω TLS και Google App
Password. Δεν απαιτεί Google Cloud project, Gmail API ή ενεργοποίηση billing.
Οι αναζητήσεις παραμένουν κανονικά Gmail queries μέσω `X-GM-RAW`, ενώ η εφαρμογή
ανοίγει τα mailbox σε read-only κατάσταση και κατεβάζει τα μηνύματα χωρίς να τα
σημαδεύει ως αναγνωσμένα. Η προηγούμενη υποστήριξη OAuth παραμένει μόνο για ήδη
ρυθμισμένες εγκαταστάσεις.

Στην έκδοση 0.3.3 προστέθηκε Linux packaging (`.deb` και `.AppImage`), ασφαλής αποθήκευση Gmail μέσω Secret Service/keyring, platform-neutral άνοιγμα αρχείων και native build scripts για Windows/Linux. Το Windows installer είναι NSIS `.exe`.

Στην έκδοση 0.3.4 ολοκληρώθηκε το κοινό Windows/Linux πακέτο: προστέθηκε έλεγχος
Linux, προστασία από σάρωση των βασικών Linux system paths, ασφαλές fallback στον
φάκελο `Documents`, σωστές ενδείξεις πλατφόρμας και Windows `.cmd` launchers που
παρακάμπτουν μόνο για τη συγκεκριμένη εκτέλεση το PowerShell Execution Policy.

Το interface ανοίγει και ως απλό browser preview από το `frontend/index.html`. Εκεί εμφανίζονται μόνο ενδεικτικά δεδομένα. Τα πραγματικά αρχεία διαβάζονται αποκλειστικά μέσα από το desktop/Tauri app.

Η πρώτη σελίδα των PDF σχεδιάζεται απευθείας σε canvas με το Mozilla PDF.js
(Apache-2.0). Τα σχετικά αρχεία και η άδεια βρίσκονται στο `frontend/vendor`.

## Δομή

```text
smart-library/
├── frontend/                 Interface χωρίς εξωτερικά web requests
├── src-tauri/
│   ├── src/
│   │   ├── scanner.rs        Read-only scanner και SHA-256
│   │   ├── app_paths.rs      Κοινοί φάκελοι Windows/Linux
│   │   ├── classifier.rs     Κανόνες αρχικής ταξινόμησης
│   │   ├── extractor.rs      Τοπική εξαγωγή κειμένου PDF/text
│   │   ├── database.rs       SQLite schema και queries
│   │   ├── gmail.rs          Gmail συντονισμός και ασφαλές Attachment Inbox
│   │   ├── gmail_imap.rs     Gmail IMAP/TLS και ανάγνωση MIME attachments
│   │   ├── credential_store.rs Windows Credential Manager / Linux Secret Service
│   │   ├── commands.rs       Ασφαλές API προς το interface
│   │   └── models.rs         Τύποι δεδομένων
│   └── tauri.conf.json
├── ROADMAP.md
├── setup-windows.cmd         Εύκολη εκκίνηση χωρίς αλλαγή Execution Policy
├── verify-windows.cmd
├── run-windows.cmd
├── build-windows.cmd
├── setup-linux.sh
├── verify-linux.sh
├── run-linux.sh
└── build-linux.sh
```

## Εκτέλεση σε Windows 11

Οι παρακάτω προϋποθέσεις χρειάζονται μία φορά:

1. Microsoft Visual Studio Build Tools με το workload **Desktop development with C++**.
2. Microsoft Edge WebView2 Runtime (είναι ήδη εγκατεστημένο στα περισσότερα Windows 11).
3. Rust μέσω `rustup`.
4. Tauri CLI 2.

Ο απλούστερος τρόπος είναι να ανοίξεις Command Prompt ή PowerShell μέσα στον
βασικό φάκελο και να χρησιμοποιήσεις τα `.cmd` αρχεία:

```bat
setup-windows.cmd
verify-windows.cmd
run-windows.cmd
```

Τα `.cmd` αρχεία χρησιμοποιούν `ExecutionPolicy Bypass` μόνο για τη δική τους
εκτέλεση και δεν αλλάζουν μόνιμα τη ρύθμιση ασφαλείας των Windows.

Για Windows installer:

```bat
build-windows.cmd
```

Παράγεται NSIS `.exe` μέσα στο
`src-tauri/target/release/bundle/nsis/`.

## Εκτέλεση και πακέτα σε Linux

Σε Debian/Ubuntu:

```bash
chmod +x setup-linux.sh verify-linux.sh run-linux.sh build-linux.sh
./setup-linux.sh
./verify-linux.sh
./run-linux.sh
```

Για να δημιουργηθούν τα Linux πακέτα:

```bash
./build-linux.sh
```

Η Linux έκδοση παράγει **`.deb`** και **`.AppImage`** μέσα στο
`src-tauri/target/release/bundle/`. Για εκτέλεση του AppImage:

```bash
chmod +x Smart-Library_*.AppImage
./Smart-Library_*.AppImage
```

Το Gmail App Password αποθηκεύεται μέσω του Linux Secret Service/keyring. Η
σύνδεση Gmail χρειάζεται κανονικό desktop login session με διαθέσιμο keyring· το
`setup-linux.sh` εγκαθιστά και το `gnome-keyring` σε Debian/Ubuntu. Για AppImage
έχει ενεργοποιηθεί bundling του multimedia framework ώστε να διατηρείται η
προεπισκόπηση ήχου/βίντεο όπου υποστηρίζεται το codec.

Το Windows installer πρέπει να χτιστεί σε Windows και τα Linux πακέτα σε Linux.
Για αυτόματο native build και των δύο από το ίδιο source υπάρχει το workflow
`.github/workflows/build-installers.yml`: εκτελεί ελέγχους, δημιουργεί NSIS
`.exe`, `.deb` και `.AppImage` και τα ανεβάζει ως build artifacts.

## Σύνδεση Gmail — ρύθμιση μία φορά

Δεν χρειάζεται Google Cloud, Gmail API, OAuth JSON ή ενεργοποίηση billing.

1. Ενεργοποίησε την **Επαλήθευση σε 2 βήματα** στον λογαριασμό Google.
2. Άνοιξε τη σελίδα [Κωδικοί πρόσβασης εφαρμογής Google](https://myaccount.google.com/apppasswords).
3. Γράψε ως όνομα εφαρμογής `Smart Library` και πάτησε **Δημιουργία**.
4. Αντέγραψε τον κωδικό εφαρμογής 16 χαρακτήρων. Μην χρησιμοποιήσεις τον κανονικό κωδικό Gmail.
5. Στο Smart Library άνοιξε το γρανάζι, συμπλήρωσε τη διεύθυνση Gmail και τον κωδικό εφαρμογής και πάτησε **Σύνδεση Gmail**.
6. Πάτησε **Λήψη συνημμένων**. Το query `has:attachment` βρίσκει email με συνημμένα και δέχεται κανονικούς όρους Gmail, π.χ. `has:attachment newer_than:1y`.

Ο κωδικός εφαρμογής αποθηκεύεται ως Generic Credential στο Windows Credential
Manager ή στο ασφαλές Linux Secret Service/keyring. Δεν εμφανίζεται ξανά, δεν γράφεται στη SQLite και δεν πρέπει να σταλεί
σε μήνυμα ή screenshot. Η ανάκλησή του γίνεται οποτεδήποτε από τον λογαριασμό
Google ή από την επιλογή **Αποσύνδεση Gmail** μέσα στην εφαρμογή.

## Ασφάλεια της έκδοσης 0.3.4

Ο scanner χρησιμοποιεί `follow_links(false)`, ώστε να μην ακολουθεί symbolic links ή junction loops. Η προεπισκόπηση επιτρέπεται μόνο για paths που βρίσκονται ήδη στο SQLite index. Τα PDF και οι εικόνες φορτώνονται μόνο έως 20 MB, ενώ το κείμενο περιορίζεται στα πρώτα 512 KB.

Οι ενέργειες ανοίγματος και εμφάνισης στον file manager περνούν από τη Rust και
επιτρέπονται μόνο όταν το ακριβές path υπάρχει ήδη στο SQLite index και συνεχίζει
να είναι πραγματικό αρχείο στον δίσκο. Το frontend δεν παίρνει γενική άδεια να
ανοίγει αυθαίρετα paths.

Τα audio previews φορτώνονται μόνο έως 32 MB και τα video previews έως 40 MB.
Μεγαλύτερα αρχεία παραμένουν στο index και μπορούν να ανοίξουν εξωτερικά, χωρίς
να αντιγράφονται στη μνήμη του preview. WMA, MKV και AVI καταγράφονται και ανοίγουν
με την προεπιλεγμένη εφαρμογή, αλλά δεν αποστέλλονται στον ενσωματωμένο player.

Η SQLite βάση αποθηκεύεται στο application-data directory του χρήστη. Τα κανονικά
folder scans παραμένουν απολύτως read-only. Μόνο όταν ο χρήστης πατήσει ρητά
**Λήψη συνημμένων Gmail** δημιουργείται το ελεγχόμενο `Email Inbox`. Ισχύει όριο
32 MB ανά attachment, όριο 48 MB ανά ακατέργαστο μήνυμα και προσωρινή εγγραφή
πριν από atomic rename. Η σύνδεση γίνεται στο `imap.gmail.com:993` με TLS και
έλεγχο πιστοποιητικού. Χρησιμοποιούνται μόνο read-only εντολές IMAP και τα email
δεν τροποποιούνται ούτε σημαδεύονται ως αναγνωσμένα. Το App Password αποθηκεύεται
στο ασφαλές credential store του λειτουργικού. Η SQLite κρατά μόνο μη
μυστικό provenance.

Η εξαγωγή κειμένου γίνεται αποκλειστικά τοπικά. Τα PDF περιορίζονται στα 50 MB
για text extraction και το κείμενο που αποθηκεύεται περιορίζεται στους 250.000
χαρακτήρες ανά αρχείο. Η υπάρχουσα βάση αναβαθμίζεται επιτόπου χωρίς διαγραφή.

Τα σύγχρονα αρχεία Office διαβάζονται ως ασφαλή ZIP/XML containers, χωρίς
εκτέλεση macros και χωρίς εξαγωγή αρχείων στον δίσκο. Ισχύει όριο 50 MB στο
αρχικό Office αρχείο, 32 MB ανά επιλεγμένο XML και 64 MB συνολικά στα XML που
αναλύονται. Τα παλαιά δυαδικά `.doc`, `.xls` και `.ppt` δεν διαβάζονται ακόμη.

Η έκδοση 0.3.4 διαθέτει native packaging για Windows και Linux. Με native build
στα Windows παράγεται NSIS `.exe`, ενώ με native Linux build παράγονται `.deb`
και `.AppImage`. Η ασφαλής αποθήκευση Gmail χρησιμοποιεί Windows Credential
Manager ή Linux Secret Service/keyring αντίστοιχα. Οι ενέργειες ανοίγματος και
εμφάνισης αρχείου περνούν από το Tauri opener και στα δύο λειτουργικά. Όταν
σαρώνεται η ρίζα `/` στο Linux, παραλείπονται τα βασικά virtual/system paths όπως
`/proc`, `/sys`, `/dev`, `/run`, `/usr` και `/var`.

## Επόμενη τεχνική φάση

Μετά το Gmail ακολουθούν Yahoo OAuth/IMAP, OCR, local-AI adapter και το πραγματικό σχέδιο αντιγραφής με
preview και undo journal. Η εκτέλεση αντιγραφών παραμένει απενεργοποιημένη μέχρι
να δοκιμαστεί με ξεχωριστό test folder.
