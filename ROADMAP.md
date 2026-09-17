# Smart Library — Roadmap

## 0.1 · Safe index

- [x] Read-only folder scan
- [x] Εξαιρέσεις system/software/game folders
- [x] SQLite schema και ιστορικό scans
- [x] SHA-256 duplicate detection
- [x] Rule-based πρώτη ταξινόμηση
- [x] Confidence και review state
- [x] PDF/image/text preview
- [x] Explorer-style interface
- [x] Άνοιγμα αρχείου και εμφάνιση στον Windows Explorer ή Linux file manager
- [x] Browser demo mode

## 0.2 · Document intelligence

- [x] Εξαγωγή κειμένου από ψηφιακά PDF και text αρχεία
- [x] DOCX/XLSX/PPTX reader και προεπισκόπηση εξαγόμενου κειμένου
- [x] CSV table preview, delimiter detection και ελληνική κωδικοποίηση
- [x] DWG/CAD, audio, video, RAW image και marine-route indexing
- [x] Εσωτερικός audio player και preview μικρών συμβατών video
- [x] Αποθηκευμένα φίλτρα σάρωσης ανά ομάδα και επέκταση αρχείου
- [ ] Πραγματικός DWG/DXF renderer μέσα στο preview
- [ ] Audio metadata (τίτλος, καλλιτέχνης, διάρκεια) και waveform
- [ ] Ανάγνωση περιεχομένων ZIP/RAR/7Z χωρίς εξαγωγή
- [ ] Ανάγνωση τοπικών EML/MSG και των συνημμένων τους
- [ ] OCR για φωτογραφίες και σαρωμένα PDF
- [ ] Ανίχνευση ημερομηνίας, εκδότη, ποσού και αριθμού παραστατικού
- [x] SQLite FTS5 indexing του εξαγόμενου κειμένου
- [x] Αναζήτηση και rule-based ταξινόμηση μέσα στο περιεχόμενο
- [x] Ανίχνευση PDF που χρειάζονται OCR
- [ ] Local model adapter (Ollama/ONNX) με αυστηρό JSON schema
- [ ] Ελληνικά, Αγγλικά και Ιταλικά

## 0.3 · Gmail source

- [x] Gmail IMAP μέσω TLS με Google App Password, χωρίς Google Cloud ή billing
- [x] Read-only `EXAMINE`, `SEARCH` και `BODY.PEEK`, χωρίς αλλαγές στα email
- [x] Gmail queries μέσω `X-GM-RAW` και αυτόματη επιλογή του All Mail mailbox
- [x] App Password στο Windows Credential Manager ή Linux Secret Service, ποτέ στη SQLite
- [x] Gmail OAuth Desktop flow με PKCE, state validation και loopback callback
- [x] Υποστήριξη υπαρχουσών OAuth συνδέσεων με `gmail.readonly`
- [x] OAuth client και tokens στο ασφαλές credential store του λειτουργικού
- [x] Ελεγχόμενο Attachment Inbox με όριο μεγέθους και atomic write
- [x] Email provenance: account, message ID, sender, subject, date και filename
- [x] SHA-256 duplicate check πριν από τη φυσική εγγραφή
- [x] Gmail search query και ασφαλή batches έως 500 email
- [x] Αυτόματο local extraction, classification, preview και index μετά τη λήψη

## 0.3.4 · Windows και Linux — παρούσα έκδοση

- [x] Κοινό Tauri source για Windows 11 και Debian/Ubuntu Linux
- [x] NSIS `.exe`, Debian `.deb` και φορητό `.AppImage`
- [x] Native άνοιγμα και εμφάνιση αρχείων και στα δύο λειτουργικά
- [x] Προστασία από βασικά Linux virtual/system paths σε σάρωση της ρίζας `/`
- [x] Ξεχωριστοί έλεγχοι και build scripts για Windows/Linux
- [x] Windows `.cmd` launchers χωρίς μόνιμη αλλαγή του Execution Policy
- [x] Αυτόματο native build και των δύο λειτουργικών μέσω GitHub Actions

## 0.4 · Controlled Smart Library

- [ ] Preview όλων των προτεινόμενων αντιγραφών
- [ ] Έλεγχος ελεύθερου χώρου πριν από κάθε batch
- [ ] Copy verification με hash
- [ ] Undo journal
- [ ] Conflict handling και version suffixes
- [ ] Αυτόματη ενέργεια μόνο πάνω από όριο confidence που επιλέγει ο χρήστης

## 0.5 · Additional sources

- [ ] AI Inbox / παρακολούθηση Downloads
- [ ] Import από Android μέσω USB ή επιλεγμένου sync folder
- [ ] Yahoo Mail σύνδεση μέσω OAuth/IMAP και λήψη attachments
- [ ] OneDrive και εξωτερικοί δίσκοι

## Κανόνες που δεν αλλάζουν

1. Τα πρωτότυπα δεν διαγράφονται αυτόματα.
2. Η πρώτη πραγματική ενέργεια είναι πάντα αντιγραφή και επαλήθευση.
3. Κάθε αρχείο κρατά source path, hash και ιστορικό αποφάσεων.
4. Χαμηλό confidence σημαίνει ανθρώπινος έλεγχος.
5. Games, software και system folders μένουν εκτός document organization.
