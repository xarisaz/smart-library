import * as pdfjsLib from "./vendor/pdf.mjs";

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL("./vendor/pdf.worker.mjs", import.meta.url).toString();

let previewRequestId = 0;

const demoFiles = [
  {
    id: 1,
    path: "C:\\Users\\Χάρης\\Downloads\\CAT_Service_48382.pdf",
    name: "CAT_Service_48382.pdf",
    extension: "pdf",
    mimeType: "application/pdf",
    sizeBytes: 1843200,
    modifiedUtc: "2026-09-15T08:42:00Z",
    sha256: "2f21d5c72b25c8039e8ed3d91e6317c349207b2813fcfbed7807f76b37b53a74",
    category: "Work",
    subcategory: "Yachting / Ferretti 80 / Invoices",
    confidence: 0.96,
    proposedName: "2026-09-15_Caterpillar_Service_Invoice_48382.pdf",
    proposedRelativePath: "02_Work/Yachting/Ferretti_80/Invoices",
    sourceRoot: "C:\\Users\\Χάρης\\Downloads",
    reviewStatus: "pending",
    isDuplicate: false,
  },
  {
    id: 2,
    path: "D:\\Documents\\Ferretti\\Bessenzoni_Crane_400kg_Manual.pdf",
    name: "Bessenzoni_Crane_400kg_Manual.pdf",
    extension: "pdf",
    mimeType: "application/pdf",
    sizeBytes: 5924454,
    modifiedUtc: "2026-09-14T17:18:00Z",
    sha256: "60c31ccf9f247ae4d1e2c8f9db2416e83e42a851b0019c8a1c15b0f16269361a",
    category: "Work",
    subcategory: "Yachting / Ferretti 80 / Manuals",
    confidence: 0.98,
    proposedName: "Bessenzoni_Crane_400kg_Manual.pdf",
    proposedRelativePath: "02_Work/Yachting/Ferretti_80/Manuals",
    sourceRoot: "D:\\Documents",
    reviewStatus: "approved",
    isDuplicate: false,
  },
  {
    id: 3,
    path: "C:\\Users\\Χάρης\\Documents\\invoice_8492026.pdf",
    name: "invoice_8492026.pdf",
    extension: "pdf",
    mimeType: "application/pdf",
    sizeBytes: 428212,
    modifiedUtc: "2026-09-13T11:05:00Z",
    sha256: "91f403f938170785f9897640866a353a6f9668353869f3913a728304924b8fe4",
    category: "Finance",
    subcategory: "Invoices",
    confidence: 0.92,
    proposedName: "2026-09-13_Invoice_8492026.pdf",
    proposedRelativePath: "03_Finance/Invoices",
    sourceRoot: "C:\\Users\\Χάρης\\Documents",
    reviewStatus: "pending",
    isDuplicate: true,
  },
  {
    id: 4,
    path: "D:\\Scans\\ktimatologio_kalyvia.pdf",
    name: "ktimatologio_kalyvia.pdf",
    extension: "pdf",
    mimeType: "application/pdf",
    sizeBytes: 3101286,
    modifiedUtc: "2026-09-12T19:30:00Z",
    sha256: "91f403f938170785f9897640866a353a6f9668353869f3913a728304924b8fe4",
    category: "Property",
    subcategory: "Property documents",
    confidence: 0.88,
    proposedName: "Ktimatologio_Kalyvia_11stremmata.pdf",
    proposedRelativePath: "04_Property/Kalyvia",
    sourceRoot: "D:\\Scans",
    reviewStatus: "pending",
    isDuplicate: true,
  },
  {
    id: 5,
    path: "C:\\Users\\Χάρης\\Desktop\\engine_hours.xlsx",
    name: "engine_hours.xlsx",
    extension: "xlsx",
    mimeType: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    sizeBytes: 98304,
    modifiedUtc: "2026-09-12T08:21:00Z",
    sha256: "448391729b3530e34f91dd552b997957174900e9e770ab950721e824d2b35d38",
    category: "Work",
    subcategory: "Yachting",
    confidence: 0.84,
    proposedName: "Ferretti_80_Engine_Generator_Hours.xlsx",
    proposedRelativePath: "02_Work/Yachting/Ferretti_80/Engine_Logs",
    sourceRoot: "C:\\Users\\Χάρης\\Desktop",
    reviewStatus: "pending",
    isDuplicate: false,
  },
  {
    id: 6,
    path: "C:\\Users\\Χάρης\\Downloads\\Audi_A3_KTEO_2026.jpg",
    name: "Audi_A3_KTEO_2026.jpg",
    extension: "jpg",
    mimeType: "image/jpeg",
    sizeBytes: 2404211,
    modifiedUtc: "2026-09-10T15:46:00Z",
    sha256: "e89490e09687e7979f77f4543d9bb20e6fd103f2ca7150110014845875b722a4",
    category: "Vehicles",
    subcategory: "Vehicle documents",
    confidence: 0.89,
    proposedName: "2026_Audi_A3_KTEO.jpg",
    proposedRelativePath: "05_Vehicles/Audi_A3/KTEO",
    sourceRoot: "C:\\Users\\Χάρης\\Downloads",
    reviewStatus: "approved",
    isDuplicate: false,
  },
  {
    id: 7,
    path: "C:\\Users\\Χάρης\\Documents\\symvasi_naulosis.docx",
    name: "symvasi_naulosis.docx",
    extension: "docx",
    mimeType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    sizeBytes: 187392,
    modifiedUtc: "2026-09-09T10:12:00Z",
    sha256: "c9e8d26df614336ce2a24ca41dabd145ae77b52a327b3b286e703159752ad754",
    category: "Legal",
    subcategory: "Contracts & Legal",
    confidence: 0.86,
    proposedName: "2026_Charter_Agreement.docx",
    proposedRelativePath: "06_Contracts_Legal/Charters",
    sourceRoot: "C:\\Users\\Χάρης\\Documents",
    reviewStatus: "pending",
    isDuplicate: false,
  },
  {
    id: 8,
    path: "D:\\Camera\\IMG_4872.jpg",
    name: "IMG_4872.jpg",
    extension: "jpg",
    mimeType: "image/jpeg",
    sizeBytes: 6732840,
    modifiedUtc: "2026-09-07T18:22:00Z",
    sha256: "d6878d206852fb9360c07719140b0e1444357f750f188495b360e6e534658ca2",
    category: "Photos",
    subcategory: "Unsorted photos",
    confidence: 0.62,
    proposedName: "IMG_4872.jpg",
    proposedRelativePath: "07_Photos/_Needs_Review",
    sourceRoot: "D:\\Camera",
    reviewStatus: "pending",
    isDuplicate: false,
  },
  {
    id: 9,
    path: "D:\\Various\\scan00042.pdf",
    name: "scan00042.pdf",
    extension: "pdf",
    mimeType: "application/pdf",
    sizeBytes: 721993,
    modifiedUtc: "2026-09-04T07:52:00Z",
    sha256: "f0c8134249c440985812488146bca574958b8fe4a6581fd141b1b5b2856dc89d",
    category: "Review",
    subcategory: "Needs review",
    confidence: 0.55,
    proposedName: "scan00042.pdf",
    proposedRelativePath: "_Needs_Review",
    sourceRoot: "D:\\Various",
    reviewStatus: "pending",
    isDuplicate: false,
  },
  {
    id: 10,
    path: "D:\\Manuals\\Lofrans_Windlass_24V.pdf",
    name: "Lofrans_Windlass_24V.pdf",
    extension: "pdf",
    mimeType: "application/pdf",
    sizeBytes: 4512290,
    modifiedUtc: "2026-08-24T12:19:00Z",
    sha256: "765369e4e4e382aebc66012e651c161f70338fb78ae72e473bd646629e9c7f92",
    category: "Work",
    subcategory: "Yachting / Ferretti 80 / Manuals",
    confidence: 0.97,
    proposedName: "Lofrans_Windlass_24V_Manual.pdf",
    proposedRelativePath: "02_Work/Yachting/Ferretti_80/Manuals",
    sourceRoot: "D:\\Manuals",
    reviewStatus: "approved",
    isDuplicate: false,
  },
];

const categoryLabels = {
  All: "Όλα τα αρχεία",
  Review: "Χρειάζονται έλεγχο",
  Personal: "Προσωπικά",
  Work: "Εργασία & Σκάφη",
  Finance: "Οικονομικά",
  Property: "Ακίνητα",
  Vehicles: "Οχήματα",
  Legal: "Συμβάσεις & Νομικά",
  Photos: "Φωτογραφίες",
  Manuals: "Manuals",
  Technical: "Σχέδια & CAD",
  Media: "Ήχος & Βίντεο",
};

const categoryDescriptions = {
  Review: "Αρχεία με χαμηλότερη βεβαιότητα που χρειάζονται δική σου απόφαση.",
  Technical: "Τεχνικά σχέδια και αρχεία CAD που έχουν καταγραφεί με ασφάλεια.",
  Media: "Αρχεία ήχου και βίντεο με τοπική προεπισκόπηση όπου υποστηρίζεται.",
};

const fileFilterGroups = [
  {
    id: "documents",
    label: "Έγγραφα και κείμενο",
    description: "PDF, Word και αρχεία απλού κειμένου",
    extensions: ["pdf", "doc", "docx", "txt", "rtf", "odt", "md", "markdown"],
  },
  {
    id: "data",
    label: "Υπολογιστικά φύλλα και δεδομένα",
    description: "Excel, CSV και δομημένα αρχεία δεδομένων",
    extensions: ["xls", "xlsx", "csv", "ods", "xml", "json", "yaml", "yml", "html", "htm"],
  },
  {
    id: "presentations",
    label: "Παρουσιάσεις και ηλεκτρονικά βιβλία",
    description: "PowerPoint, OpenDocument και e-books",
    extensions: ["ppt", "pptx", "odp", "epub", "mobi"],
  },
  {
    id: "images",
    label: "Εικόνες και φωτογραφίες RAW",
    description: "Συνηθισμένες εικόνες, HEIC, TIFF και RAW κάμερας",
    extensions: ["jpg", "jpeg", "png", "webp", "gif", "avif", "tif", "tiff", "bmp", "heic", "dng", "cr2", "cr3", "nef", "arw", "orf", "rw2"],
  },
  {
    id: "cad",
    label: "Τεχνικά σχέδια και 3D",
    description: "DWG, DXF, STEP, IGES και τρισδιάστατα μοντέλα",
    extensions: ["dwg", "dxf", "dwf", "dwt", "step", "stp", "iges", "igs", "stl", "3mf", "obj"],
  },
  {
    id: "audio",
    label: "Αρχεία ήχου",
    description: "Μουσική, ηχογραφήσεις και ασυμπίεστος ήχος",
    extensions: ["mp3", "wav", "flac", "m4a", "aac", "ogg", "opus", "wma"],
  },
  {
    id: "video",
    label: "Αρχεία βίντεο",
    description: "Συνηθισμένα formats βίντεο και containers",
    extensions: ["mp4", "m4v", "mov", "mkv", "avi", "webm"],
  },
  {
    id: "navigation",
    label: "Ναυσιπλοΐα και διαδρομές",
    description: "GPS routes, χάρτες και δεδομένα NMEA",
    extensions: ["gpx", "kml", "kmz", "nmea"],
  },
  {
    id: "archives",
    label: "Συμπιεσμένα αρχεία",
    description: "Αρχεία ZIP, RAR και 7Z",
    extensions: ["zip", "rar", "7z"],
  },
  {
    id: "email",
    label: "Τοπικά αρχεία email",
    description: "Αποθηκευμένα μηνύματα EML και MSG",
    extensions: ["eml", "msg"],
  },
];

const knownFileExtensions = fileFilterGroups.flatMap((group) => group.extensions);

const state = {
  isTauri: Boolean(window.__TAURI__?.core?.invoke),
  allFiles: [],
  visibleFiles: [],
  selectedId: null,
  category: "All",
  search: "",
  fileFilterSettings: null,
  pendingEnabledExtensions: new Set(),
  gmailStatus: {
    platformSupported: true,
    oauthConfigured: false,
    connected: false,
    authMethod: null,
    accountEmail: null,
    connectedUtc: null,
    lastSyncUtc: null,
    importedAttachments: 0,
  },
  status: {
    safeMode: true,
    indexedFiles: 0,
    pendingReview: 0,
    duplicates: 0,
    sourceCount: 0,
    smartLibraryPath: "Documents\\Smart Library",
  },
};

const elements = {
  search: document.querySelector("#global-search"),
  settingsButton: document.querySelector("#settings-button"),
  scanButton: document.querySelector("#scan-button"),
  fileList: document.querySelector("#file-list"),
  emptyState: document.querySelector("#empty-state"),
  resultCount: document.querySelector("#result-count"),
  sectionTitle: document.querySelector("#section-title"),
  sectionDescription: document.querySelector("#section-description"),
  activeSource: document.querySelector("#active-source"),
  indexedStat: document.querySelector("#indexed-stat"),
  reviewStat: document.querySelector("#review-stat"),
  duplicateStat: document.querySelector("#duplicate-stat"),
  sourceStat: document.querySelector("#source-stat"),
  libraryPath: document.querySelector("#library-path"),
  runtimeStatus: document.querySelector("#runtime-status"),
  previewEmpty: document.querySelector("#preview-empty"),
  previewContent: document.querySelector("#preview-content"),
  previewPanel: document.querySelector("#preview-panel"),
  documentPreview: document.querySelector("#document-preview"),
  selectedFileIcon: document.querySelector("#selected-file-icon"),
  selectedName: document.querySelector("#selected-name"),
  selectedMeta: document.querySelector("#selected-meta"),
  selectedPath: document.querySelector("#selected-path"),
  emailProvenance: document.querySelector("#email-provenance"),
  emailProvenanceAccount: document.querySelector("#email-provenance-account"),
  emailProvenanceContent: document.querySelector("#email-provenance-content"),
  openFileButton: document.querySelector("#open-file-button"),
  revealFileButton: document.querySelector("#reveal-file-button"),
  confidenceBadge: document.querySelector("#confidence-badge"),
  confidenceTrack: document.querySelector("#confidence-track-value"),
  analysisCategory: document.querySelector("#analysis-category"),
  analysisSubcategory: document.querySelector("#analysis-subcategory"),
  extractionStatus: document.querySelector("#extraction-status"),
  contentSnippet: document.querySelector("#content-snippet"),
  proposedLocation: document.querySelector("#proposed-location"),
  proposedName: document.querySelector("#proposed-name"),
  closePreview: document.querySelector("#close-preview"),
  approveButton: document.querySelector("#approve-button"),
  skipButton: document.querySelector("#skip-button"),
  scanOverlay: document.querySelector("#scan-overlay"),
  scanTitle: document.querySelector("#scan-title"),
  scanMessage: document.querySelector("#scan-message"),
  settingsOverlay: document.querySelector("#settings-overlay"),
  settingsCloseButton: document.querySelector("#settings-close-button"),
  settingsCancelButton: document.querySelector("#settings-cancel-button"),
  enableAllFilters: document.querySelector("#enable-all-filters"),
  resetFilters: document.querySelector("#reset-filters"),
  filterGroups: document.querySelector("#filter-groups"),
  filterCount: document.querySelector("#filter-count"),
  saveFiltersButton: document.querySelector("#save-filters-button"),
  gmailStatusPill: document.querySelector("#gmail-status-pill"),
  gmailAccountLabel: document.querySelector("#gmail-account-label"),
  gmailStatusDetail: document.querySelector("#gmail-status-detail"),
  gmailCredentialsRow: document.querySelector("#gmail-credentials-row"),
  gmailEmail: document.querySelector("#gmail-email"),
  gmailAppPassword: document.querySelector("#gmail-app-password"),
  gmailConnectButton: document.querySelector("#gmail-connect-button"),
  gmailDisconnectButton: document.querySelector("#gmail-disconnect-button"),
  gmailQuery: document.querySelector("#gmail-query"),
  gmailMaxMessages: document.querySelector("#gmail-max-messages"),
  gmailImportButton: document.querySelector("#gmail-import-button"),
  toast: document.querySelector("#toast"),
};

async function invoke(command, args = {}) {
  if (!state.isTauri) throw new Error("Tauri runtime is not available in browser preview.");
  return window.__TAURI__.core.invoke(command, args);
}

async function initialize() {
  bindEvents();

  if (state.isTauri) {
    elements.runtimeStatus.textContent = "Local engine ενεργό — τα δεδομένα μένουν στον υπολογιστή";
    try {
      state.status = await invoke("initialize_app");
      await refreshFiles();
    } catch (error) {
      showToast(normalizeError(error), true);
    }
  } else {
    state.allFiles = demoFiles.map((file) => ({
      extractionStatus: file.extension === "pdf" ? "extracted" : "unsupported",
      textPreview: file.extension === "pdf" ? "Το περιεχόμενο του εγγράφου διαβάστηκε τοπικά για την ταξινόμηση." : "",
      ...file,
    }));
    state.status = statusFromFiles(state.allFiles);
    state.status.smartLibraryPath = "C:\\Users\\Χάρης\\Documents\\Smart Library";
    elements.runtimeStatus.textContent = "Interactive preview — Safe Mode";
    applyClientFilters();
  }

  updateStatus();
  renderNavigationCounts();
  renderFiles();
}

function bindEvents() {
  // Use one permanent handler on the table body. The rows are rebuilt after
  // scans and filters, so delegation is more reliable than listeners on rows.
  elements.fileList.addEventListener("pointerdown", (event) => {
    if (event.button !== 0 || !(event.target instanceof Element)) return;
    const row = event.target.closest("tr[data-file-id]");
    if (!row || !elements.fileList.contains(row)) return;
    void selectFile(Number(row.dataset.fileId));
  });

  elements.fileList.addEventListener("keydown", (event) => {
    if (!['Enter', ' '].includes(event.key) || !(event.target instanceof Element)) return;
    const row = event.target.closest("tr[data-file-id]");
    if (!row) return;
    event.preventDefault();
    void selectFile(Number(row.dataset.fileId));
  });

  document.querySelectorAll(".nav-item").forEach((button) => {
    button.addEventListener("click", async () => {
      state.category = button.dataset.category;
      state.selectedId = null;
      document.querySelectorAll(".nav-item").forEach((item) => item.classList.remove("active"));
      button.classList.add("active");
      await refreshFiles();
      updateSectionHeading();
      hidePreview();
    });
  });

  let searchTimer;
  elements.search.addEventListener("input", () => {
    window.clearTimeout(searchTimer);
    searchTimer = window.setTimeout(async () => {
      state.search = elements.search.value.trim();
      state.selectedId = null;
      await refreshFiles();
      hidePreview();
    }, 170);
  });

  document.addEventListener("keydown", (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      elements.search.focus();
    }
    if (event.key === "Escape" && !elements.settingsOverlay.hidden) {
      closeFileFilterSettings();
    }
  });

  elements.scanButton.addEventListener("click", startScan);
  elements.settingsButton.addEventListener("click", openFileFilterSettings);
  elements.settingsCloseButton.addEventListener("click", closeFileFilterSettings);
  elements.settingsCancelButton.addEventListener("click", closeFileFilterSettings);
  elements.settingsOverlay.addEventListener("pointerdown", (event) => {
    if (event.target === elements.settingsOverlay) closeFileFilterSettings();
  });
  elements.enableAllFilters.addEventListener("click", () => {
    state.pendingEnabledExtensions = new Set(state.fileFilterSettings?.supportedExtensions || knownFileExtensions);
    updateFileFilterControls();
  });
  elements.resetFilters.addEventListener("click", () => {
    state.pendingEnabledExtensions = new Set(state.fileFilterSettings?.supportedExtensions || knownFileExtensions);
    updateFileFilterControls();
  });
  elements.filterGroups.addEventListener("change", handleFileFilterChange);
  elements.saveFiltersButton.addEventListener("click", saveFileFilters);
  elements.gmailConnectButton.addEventListener("click", connectGmail);
  elements.gmailDisconnectButton.addEventListener("click", disconnectGmail);
  elements.gmailImportButton.addEventListener("click", importGmailAttachments);
  elements.closePreview.addEventListener("click", () => {
    state.selectedId = null;
    renderFiles();
    hidePreview();
  });
  elements.approveButton.addEventListener("click", () => setReviewStatus("approved"));
  elements.skipButton.addEventListener("click", () => setReviewStatus("skipped"));
  elements.openFileButton.addEventListener("click", () => runSelectedFileAction("open_indexed_file"));
  elements.revealFileButton.addEventListener("click", () => runSelectedFileAction("reveal_indexed_file"));
}

async function openFileFilterSettings() {
  try {
    const [settings, gmailStatus] = state.isTauri
      ? await Promise.all([
        invoke("get_file_filter_settings"),
        invoke("gmail_status"),
      ])
      : [
        { supportedExtensions: [...knownFileExtensions], enabledExtensions: [...knownFileExtensions] },
        { ...state.gmailStatus, platformSupported: false },
      ];
    state.fileFilterSettings = settings;
    state.pendingEnabledExtensions = new Set(settings.enabledExtensions);
    state.gmailStatus = gmailStatus;
    renderFileFilterSettings();
    renderGmailSettings();
    elements.settingsOverlay.hidden = false;
    elements.settingsCloseButton.focus();
  } catch (error) {
    showToast(normalizeError(error), true);
  }
}

function renderGmailSettings() {
  const gmail = state.gmailStatus;
  elements.gmailStatusPill.className = "account-status-pill";

  if (!gmail.platformSupported) {
    elements.gmailStatusPill.textContent = state.isTauri ? "Μη διαθέσιμο" : "Preview";
    elements.gmailAccountLabel.textContent = "Η ασφαλής σύνδεση Gmail υποστηρίζεται σε Windows και Linux.";
    elements.gmailStatusDetail.textContent = "Ο κωδικός εφαρμογής αποθηκεύεται στο ασφαλές keyring του λειτουργικού.";
  } else if (gmail.connected) {
    elements.gmailStatusPill.textContent = "Συνδεδεμένο";
    elements.gmailStatusPill.classList.add("connected");
    elements.gmailAccountLabel.textContent = gmail.accountEmail || "Λογαριασμός Gmail";
    const imported = Number(gmail.importedAttachments || 0).toLocaleString("el-GR");
    elements.gmailStatusDetail.textContent = gmail.lastSyncUtc
      ? `${imported} συνημμένα καταγεγραμμένα · τελευταίος έλεγχος ${formatDateTime(gmail.lastSyncUtc)}`
      : `${imported} συνημμένα καταγεγραμμένα · δεν έχει γίνει λήψη ακόμη`;
  } else {
    elements.gmailStatusPill.textContent = "Δεν έχει ρυθμιστεί";
    elements.gmailAccountLabel.textContent = "Ασφαλής λήψη μόνο των συνημμένων";
    elements.gmailStatusDetail.textContent = "Γράψε το Gmail και τον κωδικό εφαρμογής 16 χαρακτήρων της Google.";
  }

  const canUseGmail = Boolean(gmail.platformSupported);
  elements.gmailCredentialsRow.hidden = Boolean(gmail.connected);
  elements.gmailEmail.disabled = !canUseGmail || Boolean(gmail.connected);
  elements.gmailAppPassword.disabled = !canUseGmail || Boolean(gmail.connected);
  elements.gmailConnectButton.disabled = !canUseGmail || Boolean(gmail.connected);
  elements.gmailDisconnectButton.hidden = !gmail.connected;
  elements.gmailImportButton.disabled = !gmail.connected;
  elements.gmailQuery.disabled = !gmail.connected;
  elements.gmailMaxMessages.disabled = !gmail.connected;
}

async function connectGmail() {
  if (!state.isTauri || !state.gmailStatus.platformSupported) return;
  const email = elements.gmailEmail.value.trim();
  const appPassword = elements.gmailAppPassword.value;
  if (!email || appPassword.replace(/\s/g, "").length !== 16) {
    showToast("Γράψε τη διεύθυνση Gmail και τον κωδικό εφαρμογής 16 χαρακτήρων.", true);
    return;
  }
  showScanOverlay(
    "Ελέγχω τη σύνδεση στο Gmail μέσω ασφαλούς IMAP…",
    "Σύνδεση Gmail",
  );
  try {
    state.gmailStatus = await invoke("connect_gmail_imap", { email, appPassword });
    renderGmailSettings();
    showToast(`Το ${state.gmailStatus.accountEmail || "Gmail"} συνδέθηκε με ασφαλή λειτουργία ανάγνωσης.`);
  } catch (error) {
    showToast(normalizeError(error), true);
  } finally {
    elements.gmailAppPassword.value = "";
    hideScanOverlay();
  }
}

async function disconnectGmail() {
  if (!state.isTauri || !state.gmailStatus.connected) return;
  const accepted = window.confirm("Να αφαιρεθεί η ασφαλής σύνδεση Gmail από αυτόν τον υπολογιστή; Τα συνημμένα που έχουν ήδη ληφθεί θα παραμείνουν.");
  if (!accepted) return;
  const previousEmail = state.gmailStatus.accountEmail || "";
  try {
    state.gmailStatus = await invoke("disconnect_gmail");
    elements.gmailEmail.value = previousEmail;
    elements.gmailAppPassword.value = "";
    renderGmailSettings();
    showToast("Η σύνδεση Gmail αφαιρέθηκε. Τα ήδη ληφθέντα αρχεία δεν άλλαξαν.");
  } catch (error) {
    showToast(normalizeError(error), true);
  }
}

async function importGmailAttachments() {
  if (!state.isTauri || !state.gmailStatus.connected) return;
  const query = elements.gmailQuery.value.trim() || "has:attachment";
  const maxMessages = Number(elements.gmailMaxMessages.value || 100);
  showScanOverlay(
    `Ελέγχω έως ${maxMessages.toLocaleString("el-GR")} email, χωρίς να αλλάζω τίποτα στο Gmail…`,
    "Λήψη συνημμένων Gmail",
  );
  try {
    const summary = await invoke("import_gmail_attachments", { query, maxMessages });
    await refreshFiles();
    state.gmailStatus = await invoke("gmail_status");
    renderGmailSettings();
    showToast(
      `${summary.imported.toLocaleString("el-GR")} νέα συνημμένα · ${summary.duplicates.toLocaleString("el-GR")} διπλότυπα · ${summary.skipped.toLocaleString("el-GR")} παραλείφθηκαν · ${summary.failed.toLocaleString("el-GR")} αποτυχίες.`,
      summary.failed > 0,
    );
  } catch (error) {
    showToast(normalizeError(error), true);
  } finally {
    hideScanOverlay();
  }
}

function closeFileFilterSettings() {
  elements.gmailAppPassword.value = "";
  elements.settingsOverlay.hidden = true;
  elements.settingsButton.focus();
}

function renderFileFilterSettings() {
  const supported = new Set(state.fileFilterSettings?.supportedExtensions || knownFileExtensions);
  const grouped = new Set(fileFilterGroups.flatMap((group) => group.extensions));
  const groups = fileFilterGroups
    .map((group) => ({ ...group, extensions: group.extensions.filter((extension) => supported.has(extension)) }))
    .filter((group) => group.extensions.length > 0);
  const ungrouped = [...supported].filter((extension) => !grouped.has(extension));
  if (ungrouped.length > 0) {
    groups.push({
      id: "other",
      label: "Άλλοι υποστηριζόμενοι τύποι",
      description: "Νέοι τύποι αρχείων που υποστηρίζει αυτή η έκδοση",
      extensions: ungrouped,
    });
  }

  elements.filterGroups.innerHTML = groups.map((group) => `
    <section class="filter-group-card" data-filter-group-card="${escapeHtml(group.id)}">
      <label class="filter-group-heading">
        <input type="checkbox" data-filter-group="${escapeHtml(group.id)}" />
        <span class="filter-checkbox" aria-hidden="true"></span>
        <span>
          <strong>${escapeHtml(group.label)}</strong>
          <small>${escapeHtml(group.description)}</small>
        </span>
      </label>
      <div class="extension-grid">
        ${group.extensions.map((extension) => `
          <label class="extension-filter">
            <input type="checkbox" value="${escapeHtml(extension)}" data-filter-extension />
            <span>.${escapeHtml(extension)}</span>
          </label>
        `).join("")}
      </div>
    </section>
  `).join("");
  updateFileFilterControls();
}

function handleFileFilterChange(event) {
  if (!(event.target instanceof HTMLInputElement)) return;

  if (event.target.matches("[data-filter-group]")) {
    const card = event.target.closest("[data-filter-group-card]");
    card?.querySelectorAll("[data-filter-extension]").forEach((input) => {
      input.checked = event.target.checked;
      if (input.checked) state.pendingEnabledExtensions.add(input.value);
      else state.pendingEnabledExtensions.delete(input.value);
    });
  } else if (event.target.matches("[data-filter-extension]")) {
    if (event.target.checked) state.pendingEnabledExtensions.add(event.target.value);
    else state.pendingEnabledExtensions.delete(event.target.value);
  }

  updateFileFilterControls();
}

function updateFileFilterControls() {
  elements.filterGroups.querySelectorAll("[data-filter-extension]").forEach((input) => {
    input.checked = state.pendingEnabledExtensions.has(input.value);
  });

  elements.filterGroups.querySelectorAll("[data-filter-group-card]").forEach((card) => {
    const groupInput = card.querySelector("[data-filter-group]");
    const extensionInputs = [...card.querySelectorAll("[data-filter-extension]")];
    const selectedCount = extensionInputs.filter((input) => input.checked).length;
    groupInput.checked = selectedCount === extensionInputs.length;
    groupInput.indeterminate = selectedCount > 0 && selectedCount < extensionInputs.length;
    card.classList.toggle("active", selectedCount > 0);
  });

  const activeCount = state.pendingEnabledExtensions.size;
  elements.filterCount.textContent = `${activeCount.toLocaleString("el-GR")} ${activeCount === 1 ? "τύπος ενεργός" : "τύποι ενεργοί"}`;
  elements.saveFiltersButton.disabled = activeCount === 0;
}

async function saveFileFilters() {
  const enabledExtensions = [...state.pendingEnabledExtensions];
  if (enabledExtensions.length === 0) {
    showToast("Επίλεξε τουλάχιστον έναν τύπο αρχείου.", true);
    return;
  }

  const previousLabel = elements.saveFiltersButton.textContent;
  elements.saveFiltersButton.disabled = true;
  elements.saveFiltersButton.textContent = "Αποθήκευση…";
  try {
    state.fileFilterSettings = state.isTauri
      ? await invoke("save_file_filter_settings", { enabledExtensions })
      : { supportedExtensions: [...knownFileExtensions], enabledExtensions };
    state.pendingEnabledExtensions = new Set(state.fileFilterSettings.enabledExtensions);
    closeFileFilterSettings();
    showToast("Τα φίλτρα αποθηκεύτηκαν και θα εφαρμοστούν στην επόμενη σάρωση.");
  } catch (error) {
    showToast(normalizeError(error), true);
  } finally {
    elements.saveFiltersButton.textContent = previousLabel;
    elements.saveFiltersButton.disabled = state.pendingEnabledExtensions.size === 0;
  }
}

async function refreshFiles() {
  if (state.isTauri) {
    try {
      state.allFiles = await invoke("list_files", {
        search: null,
        category: "All",
        limit: 50000,
      });
      if (state.search || state.category !== "All") {
        state.visibleFiles = await invoke("list_files", {
          search: state.search || null,
          category: state.category,
          limit: 50000,
        });
      } else {
        state.visibleFiles = [...state.allFiles];
      }
      state.status = await invoke("initialize_app");
    } catch (error) {
      showToast(normalizeError(error), true);
      state.visibleFiles = [];
    }
  } else {
    applyClientFilters();
  }

  updateStatus();
  renderNavigationCounts();
  renderFiles();
}

function applyClientFilters() {
  const needle = state.search.toLocaleLowerCase("el");
  state.visibleFiles = state.allFiles.filter((file) => {
    const matchesCategory = state.category === "All" || file.category === state.category;
    const haystack = `${file.name} ${file.path} ${file.subcategory} ${file.textPreview || ""}`.toLocaleLowerCase("el");
    return matchesCategory && (!needle || haystack.includes(needle));
  });
}

function statusFromFiles(files) {
  return {
    safeMode: true,
    indexedFiles: files.length,
    pendingReview: files.filter((file) => file.reviewStatus === "pending").length,
    duplicates: files.filter((file) => file.isDuplicate).length,
    sourceCount: new Set(files.map((file) => file.sourceRoot)).size,
    smartLibraryPath: "Documents\\Smart Library",
  };
}

function renderNavigationCounts() {
  const source = state.allFiles;
  const counts = { All: source.length };
  Object.keys(categoryLabels).forEach((category) => {
    if (category !== "All") counts[category] = source.filter((file) => file.category === category).length;
  });

  document.querySelectorAll("[data-count]").forEach((element) => {
    const category = element.dataset.count;
    element.textContent = compactNumber(counts[category] || 0);
  });
}

function renderFiles() {
  elements.fileList.replaceChildren();
  elements.emptyState.hidden = state.visibleFiles.length > 0;
  elements.resultCount.textContent = `${state.visibleFiles.length.toLocaleString("el-GR")} ${state.visibleFiles.length === 1 ? "αρχείο" : "αρχεία"}`;

  for (const file of state.visibleFiles) {
    const row = document.createElement("tr");
    row.className = `file-row${file.id === state.selectedId ? " selected" : ""}`;
    row.dataset.fileId = String(file.id);
    row.tabIndex = 0;
    row.setAttribute("role", "button");
    row.setAttribute("aria-label", `Άνοιγμα προεπισκόπησης ${file.name}`);
    const confidence = Math.round(file.confidence * 100);
    const lowClass = confidence < 75 ? " low-confidence" : "";
    const extension = (file.extension || "file").toLowerCase();
    const parentPath = parentFolder(file.path);

    row.innerHTML = `
      <td>
        <div class="file-name-cell">
          <div class="file-type-icon ${escapeHtml(extension)}">${escapeHtml(extension.slice(0, 4).toUpperCase())}</div>
          <div class="file-primary">
            <strong title="${escapeHtml(file.name)}">${escapeHtml(file.name)}</strong>
            <span title="${escapeHtml(parentPath)}">${escapeHtml(parentPath)}</span>
          </div>
        </div>
      </td>
      <td><span class="category-tag" title="${escapeHtml(file.subcategory)}">${escapeHtml(shortCategory(file))}</span></td>
      <td>${formatBytes(file.sizeBytes)}</td>
      <td>
        <div class="confidence-cell${lowClass}">
          <div class="mini-track"><span style="width:${confidence}%"></span></div>
          <span>${confidence}%</span>
        </div>
      </td>
      <td>
        <div class="review-state ${escapeHtml(file.reviewStatus)}" title="${reviewTitle(file.reviewStatus)}"></div>
        ${file.isDuplicate ? '<span class="duplicate-marker" title="Πιθανό διπλότυπο">2×</span>' : ""}
      </td>`;

    elements.fileList.appendChild(row);
  }
}

async function selectFile(fileId) {
  const file = state.visibleFiles.find((item) => Number(item.id) === Number(fileId));
  if (!file) {
    showToast("Δεν μπόρεσα να επιλέξω αυτό το αρχείο.", true);
    return;
  }

  const requestId = ++previewRequestId;

  try {
    state.selectedId = file.id;
    renderFiles();
    elements.previewEmpty.hidden = true;
    elements.previewContent.hidden = false;
    populateFileDetails(file);
    renderPlaceholderPreview(file);
    elements.previewPanel.scrollTop = 0;
  } catch (error) {
    showToast(`Σφάλμα προεπισκόπησης: ${normalizeError(error)}`, true);
    return;
  }

  if (state.isTauri) {
    try {
      const [preview, provenance] = await Promise.all([
        invoke("preview_file", { path: file.path }),
        invoke("get_file_provenance", { path: file.path }).catch(() => []),
      ]);
      if (state.selectedId === file.id && requestId === previewRequestId) {
        await renderNativePreview(preview, file, requestId);
        renderEmailProvenance(provenance);
      }
    } catch (error) {
      if (requestId === previewRequestId) renderPreviewMessage(normalizeError(error));
    }
  }
}

function populateFileDetails(file) {
  const extension = (file.extension || "FILE").toUpperCase();
  const confidence = Math.round(file.confidence * 100);
  elements.selectedFileIcon.textContent = extension.slice(0, 4);
  elements.selectedFileIcon.className = `large-file-icon ${file.extension.toLowerCase()}`;
  elements.selectedName.textContent = file.name;
  elements.selectedMeta.textContent = `${extension} • ${formatBytes(file.sizeBytes)} • ${formatDate(file.modifiedUtc)}`;
  elements.selectedPath.textContent = displayPath(file.path);
  elements.selectedPath.title = displayPath(file.path);
  renderEmailProvenance([]);
  elements.confidenceBadge.textContent = `${confidence}%`;
  elements.confidenceBadge.classList.toggle("low", confidence < 75);
  elements.confidenceTrack.style.width = `${confidence}%`;
  elements.analysisCategory.textContent = categoryLabels[file.category] || file.category;
  elements.analysisSubcategory.textContent = file.subcategory;
  const extraction = extractionDetails(file.extractionStatus, file.extension);
  elements.extractionStatus.textContent = extraction.label;
  elements.extractionStatus.className = `extraction-status ${extraction.className}`;
  const snippet = String(file.textPreview || "").trim();
  elements.contentSnippet.hidden = snippet.length === 0;
  elements.contentSnippet.textContent = snippet;
  elements.proposedLocation.textContent = `Smart Library/${file.proposedRelativePath}`;
  elements.proposedLocation.title = elements.proposedLocation.textContent;
  elements.proposedName.textContent = file.proposedName;
  elements.proposedName.title = file.proposedName;
  elements.approveButton.textContent = file.reviewStatus === "approved" ? "Η πρόταση εγκρίθηκε" : "Έγκριση πρότασης";
}

function renderEmailProvenance(items) {
  const provenance = Array.isArray(items) ? items : [];
  if (provenance.length === 0) {
    elements.emailProvenance.hidden = true;
    elements.emailProvenanceAccount.textContent = "";
    elements.emailProvenanceContent.replaceChildren();
    return;
  }

  const first = provenance[0];
  elements.emailProvenance.hidden = false;
  elements.emailProvenanceAccount.textContent = first.accountEmail || "Gmail";
  elements.emailProvenanceContent.innerHTML = `
    <strong title="${escapeHtml(first.subject || "")}">${escapeHtml(first.subject || "Χωρίς θέμα")}</strong>
    <span title="${escapeHtml(first.sender || "")}">Από: ${escapeHtml(first.sender || "Άγνωστος αποστολέας")}</span>
    <span>${escapeHtml(first.messageDate || "Ημερομηνία άγνωστη")} · αρχικό όνομα: ${escapeHtml(first.originalFilename || "—")}</span>
    ${provenance.length > 1 ? `<span class="email-provenance-more">Υπάρχουν ${provenance.length - 1} ακόμη εγγραφές προέλευσης για το ίδιο αρχείο.</span>` : ""}
  `;
}

function renderPlaceholderPreview(file) {
  const extension = file.extension.toLowerCase();
  if (extension === "pdf") {
    elements.documentPreview.innerHTML = `
      <div class="paper-preview" aria-label="Ενδεικτική προεπισκόπηση εγγράφου">
        <div class="paper-mark"></div>
        <div class="paper-title">DOCUMENT PREVIEW</div>
        <div class="paper-line"></div><div class="paper-line"></div><div class="paper-line short"></div>
        <div class="paper-line"></div><div class="paper-line tiny"></div>
        <div class="paper-total">TOTAL &nbsp; € —</div>
      </div>`;
  } else if (["xlsx", "xls", "csv"].includes(extension)) {
    elements.documentPreview.innerHTML = `
      <div class="paper-preview" aria-label="Ενδεικτική προεπισκόπηση πίνακα">
        <div class="paper-mark"></div>
        <div class="paper-title">SPREADSHEET</div>
        <div class="paper-line"></div><div class="paper-line"></div><div class="paper-line"></div>
        <div class="paper-line"></div><div class="paper-line short"></div>
      </div>`;
  } else if (["docx", "pptx"].includes(extension)) {
    elements.documentPreview.innerHTML = `
      <div class="paper-preview" aria-label="Ενδεικτική προεπισκόπηση εγγράφου Office">
        <div class="paper-mark"></div>
        <div class="paper-title">OFFICE DOCUMENT</div>
        <div class="paper-line"></div><div class="paper-line"></div><div class="paper-line short"></div>
        <div class="paper-line"></div><div class="paper-line tiny"></div>
      </div>`;
  } else if (isAudioExtension(extension)) {
    elements.documentPreview.innerHTML = `
      <div class="media-placeholder" aria-label="Φόρτωση αρχείου ήχου">
        <div class="media-disc">♪</div>
        <div class="media-bars"><span></span><span></span><span></span><span></span><span></span></div>
        <strong>Φόρτωση ήχου…</strong>
      </div>`;
  } else if (isVideoExtension(extension)) {
    elements.documentPreview.innerHTML = `
      <div class="media-placeholder" aria-label="Φόρτωση αρχείου βίντεο">
        <div class="media-disc">▶</div>
        <strong>Φόρτωση βίντεο…</strong>
      </div>`;
  } else if (isCadExtension(extension)) {
    elements.documentPreview.innerHTML = `
      <div class="cad-placeholder" aria-label="Αρχείο τεχνικού σχεδίου">
        <svg viewBox="0 0 80 80" aria-hidden="true"><path d="M10 10h60v60H10zM18 58l13-35 12 35 9-22 10 22M18 63h44"/></svg>
        <strong>${escapeHtml(extension.toUpperCase())} · Τεχνικό σχέδιο</strong>
        <span>Άνοιγμα με το εγκατεστημένο CAD πρόγραμμα</span>
      </div>`;
  } else {
    renderPreviewMessage("Η πραγματική προεπισκόπηση εμφανίζεται όταν η εφαρμογή ανοίξει ως desktop app.");
  }
}

async function renderNativePreview(preview, file, requestId) {
  elements.documentPreview.replaceChildren();
  if (preview.kind === "image" && preview.dataUrl) {
    const image = document.createElement("img");
    image.src = preview.dataUrl;
    image.alt = `Προεπισκόπηση ${file.name}`;
    elements.documentPreview.appendChild(image);
  } else if (preview.kind === "pdf" && preview.dataUrl) {
    await renderPdfPreview(preview.dataUrl, file, requestId);
  } else if (preview.kind === "audio" && preview.dataUrl) {
    renderMediaPreview("audio", preview.dataUrl, file);
  } else if (preview.kind === "video" && preview.dataUrl) {
    renderMediaPreview("video", preview.dataUrl, file);
  } else if (preview.kind === "text") {
    const extension = file.extension.toLowerCase();
    if (extension === "csv") {
      renderCsvPreview(preview.text || "", preview.message);
      return;
    }

    const pre = document.createElement("pre");
    pre.textContent = preview.text || "";
    if (["docx", "xlsx", "pptx"].includes(extension)) {
      const badge = document.createElement("span");
      badge.className = "office-text-badge";
      badge.textContent = "Εξαγόμενο κείμενο";
      badge.title = preview.message || badge.textContent;
      elements.documentPreview.appendChild(badge);
    }
    elements.documentPreview.appendChild(pre);
  } else {
    renderPreviewMessage(preview.message || "Δεν υπάρχει προεπισκόπηση για αυτόν τον τύπο αρχείου.");
  }
}

function renderMediaPreview(kind, dataUrl, file) {
  const wrapper = document.createElement("div");
  wrapper.className = `native-media-preview ${kind}`;
  const media = document.createElement(kind);
  media.controls = true;
  media.preload = "metadata";
  media.src = dataUrl;
  media.setAttribute("aria-label", `${kind === "audio" ? "Αναπαραγωγή" : "Προεπισκόπηση"} ${file.name}`);
  media.addEventListener("error", () => {
    renderPreviewMessage("Ο ενσωματωμένος player δεν υποστηρίζει αυτόν τον codec. Χρησιμοποίησε «Άνοιγμα αρχείου».");
  }, { once: true });

  if (kind === "audio") {
    const artwork = document.createElement("div");
    artwork.className = "audio-artwork";
    artwork.textContent = "♪";
    wrapper.appendChild(artwork);
  }
  wrapper.appendChild(media);
  elements.documentPreview.appendChild(wrapper);
}

function isAudioExtension(extension) {
  return ["mp3", "wav", "flac", "m4a", "aac", "ogg", "opus", "wma"].includes(extension);
}

function isVideoExtension(extension) {
  return ["mp4", "m4v", "mov", "mkv", "avi", "webm"].includes(extension);
}

function isCadExtension(extension) {
  return ["dwg", "dxf", "dwf", "dwt", "step", "stp", "iges", "igs", "stl", "3mf", "obj"].includes(extension);
}

function renderCsvPreview(text, message) {
  const delimiter = detectCsvDelimiter(text);
  const parsed = parseDelimitedText(text, delimiter, 150, 30);
  if (parsed.rows.length === 0) {
    renderPreviewMessage(message || "Το CSV δεν περιέχει αναγνώσιμες γραμμές.");
    return;
  }

  const badge = document.createElement("span");
  badge.className = "office-text-badge";
  badge.textContent = `CSV · ${delimiterLabel(delimiter)}`;
  badge.title = message || "Προεπισκόπηση CSV μέσα στην εφαρμογή";

  const scroller = document.createElement("div");
  scroller.className = "csv-preview-scroll";
  const table = document.createElement("table");
  table.className = "csv-preview-table";

  parsed.rows.forEach((row, rowIndex) => {
    const tableRow = document.createElement("tr");
    row.forEach((value) => {
      const cell = document.createElement(rowIndex === 0 ? "th" : "td");
      const visibleValue = String(value).slice(0, 500);
      cell.textContent = visibleValue;
      if (String(value).length > visibleValue.length) cell.title = String(value);
      tableRow.appendChild(cell);
    });
    table.appendChild(tableRow);
  });

  scroller.appendChild(table);
  elements.documentPreview.append(badge, scroller);

  if (parsed.truncated) {
    const limitNotice = document.createElement("span");
    limitNotice.className = "csv-limit-notice";
    limitNotice.textContent = "Προβάλλονται οι πρώτες 150 γραμμές και 30 στήλες";
    elements.documentPreview.appendChild(limitNotice);
  }
}

function detectCsvDelimiter(text) {
  const candidates = [";", ",", "\t", "|"];
  const sample = String(text).split(/\r?\n/).filter((line) => line.trim()).slice(0, 8);
  let selected = ",";
  let bestScore = 0;

  candidates.forEach((candidate) => {
    const counts = sample.map((line) => countDelimiterOutsideQuotes(line, candidate));
    const positive = counts.filter((count) => count > 0);
    if (positive.length === 0) return;
    const consistency = positive.filter((count) => count === positive[0]).length;
    const score = consistency * 100 + positive.reduce((sum, count) => sum + count, 0);
    if (score > bestScore) {
      bestScore = score;
      selected = candidate;
    }
  });

  return selected;
}

function countDelimiterOutsideQuotes(line, delimiter) {
  let inQuotes = false;
  let count = 0;
  for (let index = 0; index < line.length; index += 1) {
    if (line[index] === '"') {
      if (inQuotes && line[index + 1] === '"') index += 1;
      else inQuotes = !inQuotes;
    } else if (!inQuotes && line[index] === delimiter) {
      count += 1;
    }
  }
  return count;
}

function parseDelimitedText(text, delimiter, maximumRows, maximumColumns) {
  const input = String(text).replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  const rows = [];
  let row = [];
  let field = "";
  let inQuotes = false;
  let index = 0;
  let columnsTruncated = false;

  const pushField = () => {
    if (row.length < maximumColumns) row.push(field);
    else columnsTruncated = true;
    field = "";
  };

  const pushRow = () => {
    pushField();
    if (row.some((value) => value.trim() !== "")) rows.push(row);
    row = [];
  };

  while (index < input.length && rows.length < maximumRows) {
    const character = input[index];
    if (character === '"') {
      if (inQuotes && input[index + 1] === '"') {
        field += '"';
        index += 1;
      } else {
        inQuotes = !inQuotes;
      }
    } else if (!inQuotes && character === delimiter) {
      pushField();
    } else if (!inQuotes && character === "\n") {
      pushRow();
    } else {
      field += character;
    }
    index += 1;
  }

  if (rows.length < maximumRows && (field.length > 0 || row.length > 0)) pushRow();
  return { rows, truncated: index < input.length || columnsTruncated };
}

function delimiterLabel(delimiter) {
  if (delimiter === "\t") return "tab";
  if (delimiter === ";") return "semicolon";
  if (delimiter === "|") return "pipe";
  return "comma";
}

async function renderPdfPreview(dataUrl, file, requestId) {
  elements.documentPreview.innerHTML = `
    <div class="pdf-loading" role="status">
      <span class="pdf-loading-spinner"></span>
      <span>Φόρτωση πρώτης σελίδας…</span>
    </div>`;

  try {
    const bytes = dataUrlToBytes(dataUrl);
    const loadingTask = pdfjsLib.getDocument({ data: bytes });
    const pdf = await loadingTask.promise;
    if (requestId !== previewRequestId) {
      await pdf.destroy();
      return;
    }

    const page = await pdf.getPage(1);
    const initialViewport = page.getViewport({ scale: 1 });
    const availableWidth = Math.max(elements.documentPreview.clientWidth - 24, 140);
    const availableHeight = Math.max(elements.documentPreview.clientHeight - 24, 140);
    const fitScale = Math.min(
      availableWidth / initialViewport.width,
      availableHeight / initialViewport.height,
    );
    const viewport = page.getViewport({ scale: fitScale });
    const outputScale = Math.min(window.devicePixelRatio || 1, 2);
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d", { alpha: false });
    if (!context) throw new Error("Δεν είναι διαθέσιμο το canvas renderer.");

    canvas.width = Math.floor(viewport.width * outputScale);
    canvas.height = Math.floor(viewport.height * outputScale);
    canvas.style.width = `${Math.floor(viewport.width)}px`;
    canvas.style.height = `${Math.floor(viewport.height)}px`;
    canvas.setAttribute("aria-label", `Πρώτη σελίδα του ${file.name}`);

    const wrapper = document.createElement("div");
    wrapper.className = "pdf-canvas-wrap";
    wrapper.appendChild(canvas);
    const pageBadge = document.createElement("span");
    pageBadge.className = "pdf-page-badge";
    pageBadge.textContent = `Σελίδα 1 / ${pdf.numPages}`;
    wrapper.appendChild(pageBadge);

    if (requestId !== previewRequestId) {
      await pdf.destroy();
      return;
    }
    elements.documentPreview.replaceChildren(wrapper);

    await page.render({
      canvasContext: context,
      viewport,
      transform: outputScale !== 1 ? [outputScale, 0, 0, outputScale, 0, 0] : null,
    }).promise;
    await pdf.destroy();
  } catch (error) {
    if (requestId === previewRequestId) {
      renderPreviewMessage(`Δεν μπόρεσα να σχεδιάσω το PDF: ${normalizeError(error)}`);
    }
  }
}

function dataUrlToBytes(dataUrl) {
  const separatorIndex = dataUrl.indexOf(",");
  if (separatorIndex < 0) throw new Error("Μη έγκυρα δεδομένα PDF.");
  const binary = window.atob(dataUrl.slice(separatorIndex + 1));
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}

function renderPreviewMessage(message) {
  elements.documentPreview.replaceChildren();
  const paragraph = document.createElement("p");
  paragraph.className = "preview-message";
  paragraph.textContent = message;
  elements.documentPreview.appendChild(paragraph);
}

function hidePreview() {
  previewRequestId += 1;
  renderEmailProvenance([]);
  elements.previewEmpty.hidden = false;
  elements.previewContent.hidden = true;
  elements.previewPanel.scrollTop = 0;
}

async function runSelectedFileAction(command) {
  if (state.selectedId == null) return;
  const file = state.allFiles.find((item) => Number(item.id) === Number(state.selectedId));
  if (!file) {
    showToast("Δεν βρέθηκε το επιλεγμένο αρχείο στο index.", true);
    return;
  }

  if (!state.isTauri) {
    showToast("Το άνοιγμα αρχείων λειτουργεί μέσα στην desktop εφαρμογή.");
    return;
  }

  elements.openFileButton.disabled = true;
  elements.revealFileButton.disabled = true;

  try {
    await invoke(command, { path: file.path });
    showToast(command === "open_indexed_file"
      ? `Άνοιγμα του «${file.name}» με την προεπιλεγμένη εφαρμογή.`
      : `Το «${file.name}» εμφανίστηκε στον φάκελό του.`);
  } catch (error) {
    showToast(normalizeError(error), true);
  } finally {
    elements.openFileButton.disabled = false;
    elements.revealFileButton.disabled = false;
  }
}

async function setReviewStatus(status) {
  if (state.selectedId == null) return;
  const file = state.visibleFiles.find((item) => item.id === state.selectedId);
  if (!file) return;

  try {
    if (state.isTauri) {
      await invoke("set_review_status", { fileId: file.id, status });
    }
    const sourceFile = state.allFiles.find((item) => item.id === file.id);
    if (sourceFile) sourceFile.reviewStatus = status;
    file.reviewStatus = status;
    state.status = state.isTauri ? await invoke("initialize_app") : statusFromFiles(state.allFiles);
    updateStatus();
    renderFiles();
    populateFileDetails(file);
    showToast(status === "approved"
      ? "Η πρόταση αποθηκεύτηκε. Κανένα αρχείο δεν αντιγράφηκε ή μετακινήθηκε."
      : "Το αρχείο σημειώθηκε για παράβλεψη.");
  } catch (error) {
    showToast(normalizeError(error), true);
  }
}

async function startScan() {
  if (!state.isTauri) {
    showScanOverlay("Δοκιμαστική σάρωση φακέλου Documents…");
    window.setTimeout(() => {
      hideScanOverlay();
      showToast("Demo: 10 αρχεία αναγνωρίστηκαν χωρίς καμία αλλαγή στα πρωτότυπα.");
    }, 1150);
    return;
  }

  try {
    const selectedPath = await invoke("pick_folder");
    if (!selectedPath) return;
    showScanOverlay(`Σάρωση και τοπική ανάγνωση: ${selectedPath}`);
    const summary = await invoke("scan_folder", { path: selectedPath });
    elements.activeSource.textContent = displayPath(summary.sourceRoot);
    await refreshFiles();
    showToast(
      `${summary.indexed.toLocaleString("el-GR")} αρχεία στο index · κείμενο από ${summary.textExtracted.toLocaleString("el-GR")} · OCR χρειάζονται ${summary.needsOcr.toLocaleString("el-GR")} · ${summary.duplicates.toLocaleString("el-GR")} διπλότυπα.`,
    );
  } catch (error) {
    showToast(normalizeError(error), true);
  } finally {
    hideScanOverlay();
  }
}

function showScanOverlay(message, title = "Ασφαλής σάρωση σε εξέλιξη") {
  elements.scanTitle.textContent = title;
  elements.scanMessage.textContent = message;
  elements.scanOverlay.hidden = false;
}

function hideScanOverlay() {
  elements.scanOverlay.hidden = true;
  elements.scanTitle.textContent = "Εργασία σε εξέλιξη";
}

function updateStatus() {
  elements.indexedStat.textContent = compactNumber(state.status.indexedFiles || 0);
  elements.reviewStat.textContent = compactNumber(state.status.pendingReview || 0);
  elements.duplicateStat.textContent = compactNumber(state.status.duplicates || 0);
  elements.sourceStat.textContent = `${state.status.sourceCount || 0} ${(state.status.sourceCount || 0) === 1 ? "πηγή" : "πηγές"}`;
  elements.libraryPath.textContent = state.status.smartLibraryPath || "Documents\\Smart Library";
  elements.libraryPath.title = elements.libraryPath.textContent;
}

function updateSectionHeading() {
  elements.sectionTitle.textContent = categoryLabels[state.category] || state.category;
  elements.sectionDescription.textContent = categoryDescriptions[state.category]
    || "Αρχεία που έχει αναγνωρίσει και ταξινομήσει το Smart Library.";
}

let toastTimer;
function showToast(message, isError = false) {
  window.clearTimeout(toastTimer);
  elements.toast.textContent = message;
  elements.toast.classList.toggle("error", isError);
  elements.toast.classList.add("visible");
  toastTimer = window.setTimeout(() => elements.toast.classList.remove("visible"), 4200);
}

function formatBytes(bytes) {
  const value = Number(bytes || 0);
  if (value < 1024) return `${value} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let size = value / 1024;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size >= 10 ? size.toFixed(0) : size.toFixed(1)} ${units[unitIndex]}`;
}

function formatDate(value) {
  try {
    return new Intl.DateTimeFormat("el-GR", { day: "2-digit", month: "short", year: "numeric" }).format(new Date(value));
  } catch {
    return value || "—";
  }
}

function formatDateTime(value) {
  try {
    return new Intl.DateTimeFormat("el-GR", {
      day: "2-digit",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    }).format(new Date(value));
  } catch {
    return value || "—";
  }
}

function compactNumber(value) {
  return new Intl.NumberFormat("el-GR", { notation: value > 9999 ? "compact" : "standard", maximumFractionDigits: 1 }).format(value);
}

function parentFolder(path) {
  const normalized = displayPath(path).replaceAll("/", "\\");
  const parts = normalized.split("\\");
  parts.pop();
  return parts.slice(-3).join("\\") || "—";
}

function displayPath(path) {
  return String(path || "").replace(/^\\\\\?\\/, "");
}

function shortCategory(file) {
  const parts = file.subcategory.split(" / ");
  return parts.length > 1 ? parts.slice(-2).join(" / ") : file.subcategory;
}

function reviewTitle(status) {
  if (status === "approved") return "Εγκεκριμένη πρόταση";
  if (status === "skipped") return "Παράβλεψη";
  return "Αναμονή για έλεγχο";
}

function extractionDetails(status, extension = "") {
  const normalizedExtension = String(extension).toLowerCase();
  if (status === "unsupported" && isCadExtension(normalizedExtension)) {
    return { label: "Το τεχνικό αρχείο μπήκε στο index — απαιτεί εξωτερικό CAD viewer", className: "neutral" };
  }
  if (status === "unsupported" && isAudioExtension(normalizedExtension)) {
    return { label: "Το αρχείο ήχου μπήκε στο index — διαθέσιμη αναπαραγωγή όπου υποστηρίζεται", className: "success" };
  }
  if (status === "unsupported" && isVideoExtension(normalizedExtension)) {
    return { label: "Το βίντεο μπήκε στο index — διαθέσιμη προεπισκόπηση όπου υποστηρίζεται", className: "success" };
  }

  switch (status) {
    case "extracted":
      return { label: "Το κείμενο διαβάστηκε και μπήκε στο index", className: "success" };
    case "needs_ocr":
      return { label: "Δεν βρέθηκε ψηφιακό κείμενο — χρειάζεται OCR", className: "warning" };
    case "failed":
      return { label: "Η ανάγνωση απέτυχε — το αρχείο έμεινε ασφαλές", className: "warning" };
    case "too_large":
      return { label: "Πάνω από το ασφαλές όριο ανάγνωσης των 50 MB", className: "neutral" };
    case "unsupported":
      return { label: "Η ανάγνωση αυτού του τύπου θα προστεθεί αργότερα", className: "neutral" };
    case "empty":
      return { label: "Το αρχείο δεν περιέχει αρκετό αναγνώσιμο κείμενο", className: "neutral" };
    default:
      return { label: "Κάνε ξανά σάρωση για ανάγνωση περιεχομένου", className: "neutral" };
  }
}

function normalizeError(error) {
  if (typeof error === "string") return error;
  if (error?.message) return error.message;
  return "Παρουσιάστηκε άγνωστο σφάλμα.";
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

initialize();
