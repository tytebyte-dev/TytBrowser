package dev.tytebyte.tytbrowser

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.app.AlertDialog
import android.app.DownloadManager
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.Color
import android.net.Uri
import android.net.http.SslError
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.text.Editable
import android.text.InputType
import android.text.TextWatcher
import android.view.ContextThemeWrapper
import android.view.Gravity
import android.view.KeyEvent
import android.view.View
import android.view.ViewGroup
import android.view.WindowManager
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import android.webkit.CookieManager
import android.webkit.DownloadListener
import android.webkit.SslErrorHandler
import android.webkit.URLUtil
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebSettings
import android.webkit.WebStorage
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.EditText
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.PopupMenu
import android.widget.ProgressBar
import android.widget.TextView
import android.widget.Toast
import dev.tytebyte.tytbrowser.core.Core
import dev.tytebyte.tytbrowser.core.InputKind
import dev.tytebyte.tytbrowser.core.START_URL
import dev.tytebyte.tytbrowser.core.Verdict
import dev.tytebyte.tytbrowser.data.Db
import dev.tytebyte.tytbrowser.data.Prefs
import dev.tytebyte.tytbrowser.ui.BookmarksHistoryPanel
import dev.tytebyte.tytbrowser.ui.DownloadsPanel
import dev.tytebyte.tytbrowser.ui.HomeView
import dev.tytebyte.tytbrowser.ui.Pal
import dev.tytebyte.tytbrowser.ui.ReaderPage
import dev.tytebyte.tytbrowser.ui.SettingsPanel
import dev.tytebyte.tytbrowser.ui.TabsPanel
import dev.tytebyte.tytbrowser.ui.divider
import dev.tytebyte.tytbrowser.ui.dp
import dev.tytebyte.tytbrowser.ui.iconButton
import dev.tytebyte.tytbrowser.ui.matchMatch
import dev.tytebyte.tytbrowser.ui.ripple
import dev.tytebyte.tytbrowser.ui.shape
import dev.tytebyte.tytbrowser.ui.text
import org.json.JSONObject
import org.json.JSONTokener
import java.io.ByteArrayInputStream
import java.util.concurrent.atomic.AtomicLong
import kotlin.concurrent.thread

class BrowserActivity : Activity() {

    private lateinit var prefs: Prefs
    private lateinit var db: Db
    private val tabs = ArrayList<Tab>()
    private lateinit var cur: Tab
    private var nextId = 1L
    private val blockedTotal = AtomicLong(0)
    private var defaultUa = ""

    private lateinit var root: FrameLayout
    private lateinit var main: LinearLayout
    private lateinit var content: FrameLayout
    private lateinit var overlay: FrameLayout
    private val panels = ArrayList<View>()
    private var readerView: View? = null
    private var readerArticle: JSONObject? = null
    private var findBar: View? = null

    // Элементы адресной строки
    private var urlField: EditText? = null
    private var progressBar: ProgressBar? = null
    private var tabsCount: TextView? = null
    private var reloadBtn: TextView? = null
    private var backBtn: TextView? = null
    private var readerBtn: TextView? = null
    private var lockView: TextView? = null
    private var suggestBox: LinearLayout? = null
    private var settingText = false

    private var customView: View? = null
    private var customCallback: WebChromeClient.CustomViewCallback? = null
    private var fileCallback: ValueCallback<Array<Uri>>? = null
    private var pendingDownload: (() -> Unit)? = null

    private var pal = Pal(1)

    // ───────────────────────── жизненный цикл ─────────────────────────

    override fun onCreate(savedInstanceState: Bundle?) {
        prefs = Prefs(this)
        setTheme(android.R.style.Theme_Material_NoActionBar)
        super.onCreate(savedInstanceState)
        db = Db(this)
        blockedTotal.set(prefs.blockedTotal)
        defaultUa = try { WebSettings.getDefaultUserAgent(this) } catch (_: Throwable) { "" }
        window.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE)

        Core.configureBlocker(prefs.blockAds, prefs.blockTrackers)
        loadUserRules()

        root = FrameLayout(this)
        main = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        content = FrameLayout(this)
        overlay = FrameLayout(this)
        root.addView(main, matchMatch())
        root.addView(overlay, matchMatch())
        setContentView(root)

        restoreTabs()
        handleIntent(intent, initial = true)
        applyChrome()
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleIntent(intent, initial = false)
    }

    override fun onResume() {
        super.onResume()
        cur.web?.onResume()
    }

    override fun onPause() {
        super.onPause()
        cur.web?.onPause()
        saveState()
    }

    override fun onDestroy() {
        if (isFinishing && prefs.clearOnExit) clearAllData()
        saveState()
        tabs.forEach { it.web?.let { w -> (w.parent as? ViewGroup)?.removeView(w); w.destroy() } }
        super.onDestroy()
    }

    override fun onTrimMemory(level: Int) {
        super.onTrimMemory(level)
        if (level >= TRIM_MEMORY_RUNNING_LOW) trimWebViews(maxAlive = 1)
    }

    private fun saveState() {
        val regular = tabs.filter { !it.priv }
        prefs.savedTabs = regular.joinToString("\n") { if (it.home) START_URL else it.url }
        prefs.savedActive = regular.indexOf(cur).coerceAtLeast(0)
        prefs.blockedTotal = blockedTotal.get()
    }

    private fun restoreTabs() {
        val lines = prefs.savedTabs.split("\n").filter { it.isNotBlank() }
        lines.forEach { u ->
            val t = Tab(nextId++, false)
            if (u != START_URL) { t.url = u; t.home = false; t.pageUrl = u }
            tabs += t
        }
        if (tabs.isEmpty()) tabs += Tab(nextId++, false)
        cur = tabs[prefs.savedActive.coerceIn(0, tabs.size - 1)]
    }

    private fun handleIntent(intent: Intent?, initial: Boolean) {
        intent ?: return
        val target: String? = when (intent.action) {
            Intent.ACTION_VIEW -> intent.data?.toString()
            Intent.ACTION_SEND -> intent.getStringExtra(Intent.EXTRA_TEXT)
            Intent.ACTION_WEB_SEARCH -> intent.getStringExtra("query")
            else -> null
        }
        if (target.isNullOrBlank()) return
        val r = Core.resolve(target, prefs.searchEngine)
        if (r.kind == InputKind.INTERNAL) return
        if (cur.home && !cur.priv && !initial) load(cur, r.url) else newTab(r.url, priv = false, foreground = true)
    }

    // ───────────────────────── вкладки ─────────────────────────

    private fun newTab(url: String?, priv: Boolean, foreground: Boolean, parent: Tab? = null) {
        val t = Tab(nextId++, priv)
        t.parentId = parent?.id ?: -1L
        tabs += t
        if (foreground) select(t)
        if (url != null) load(t, url)
        if (!foreground) toast("Вкладка открыта в фоне")
        updateTabsCount()
    }

    private fun select(t: Tab) {
        if (t !== cur) cur.web?.onPause()
        val wasPriv = cur.priv
        cur = t
        t.lastUsed = System.currentTimeMillis()
        closeFind()
        closeReader()
        if (wasPriv != t.priv) applyChrome() else { showContent(); render() }
        trimWebViews(maxAlive = 5)
    }

    private fun destroyWeb(t: Tab) {
        t.web?.let { (it.parent as? ViewGroup)?.removeView(it); it.destroy() }
        t.web = null
    }

    private fun closeTab(t: Tab) {
        val idx = tabs.indexOf(t)
        if (idx < 0) return
        tabs.remove(t)
        destroyWeb(t)
        if (tabs.none { !it.priv }) tabs += Tab(nextId++, false) // всегда остаётся хотя бы одна обычная вкладка
        if (t === cur) {
            val next = tabs.firstOrNull { it.id == t.parentId }
                ?: tabs.filter { it.priv == t.priv }.let { same -> same.getOrNull(idx.coerceAtMost(same.size - 1)) }
                ?: tabs.first { !it.priv }
            select(next)
        }
        updateTabsCount()
    }

    private fun closeAllTabs(priv: Boolean) {
        val victims = tabs.filter { it.priv == priv }
        val wasCur = cur in victims
        victims.forEach { destroyWeb(it); tabs.remove(it) }
        if (tabs.none { !it.priv }) tabs += Tab(nextId++, false)
        if (wasCur) select(tabs.first { !it.priv }) else updateTabsCount()
    }

    /** Выгружает WebView давно не использованных вкладок: адрес остаётся, страница загрузится заново при открытии. */
    private fun trimWebViews(maxAlive: Int) {
        val alive = tabs.filter { it.web != null }
        if (alive.size <= maxAlive) return
        alive.filter { it !== cur }.sortedBy { it.lastUsed }.take(alive.size - maxAlive).forEach { t ->
            t.web?.let { (it.parent as? ViewGroup)?.removeView(it); it.destroy() }
            t.web = null
        }
    }

    private fun updateTabsCount() {
        tabsCount?.text = tabs.count { it.priv == cur.priv }.toString()
    }

    // ───────────────────────── интерфейс ─────────────────────────

    private fun themed(p: Pal = pal): Context =
        ContextThemeWrapper(this, if (p.dark) android.R.style.Theme_Material else android.R.style.Theme_Material_Light)

    /** Пересобирает панели адреса/контента под тему, положение адресной строки и режим вкладки. */
    private fun applyChrome() {
        pal = Pal(prefs.theme, cur.priv)
        window.statusBarColor = pal.bar
        window.navigationBarColor = pal.bar
        var flags = 0
        if (!pal.dark) {
            if (Build.VERSION.SDK_INT >= 23) flags = flags or View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR
            if (Build.VERSION.SDK_INT >= 26) flags = flags or View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR
        }
        window.decorView.systemUiVisibility = flags
        root.setBackgroundColor(pal.bg)

        main.removeAllViews()
        (content.parent as? ViewGroup)?.removeView(content)
        val bar = buildBar()
        if (!prefs.addressBarBottom) main.addView(bar, LinearLayout.LayoutParams(-1, -2))
        main.addView(content, LinearLayout.LayoutParams(-1, 0, 1f))
        if (prefs.addressBarBottom) main.addView(bar, LinearLayout.LayoutParams(-1, -2))
        showContent()
        render()
    }

    private fun buildBar(): View {
        val ctx = themed()
        val bar = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL; setBackgroundColor(pal.bar) }
        val prog = ProgressBar(ctx, null, android.R.attr.progressBarStyleHorizontal).apply {
            max = 100
            progressDrawable.setColorFilter(pal.accent, android.graphics.PorterDuff.Mode.SRC_IN)
            visibility = View.INVISIBLE
            layoutParams = LinearLayout.LayoutParams(-1, ctx.dp(3))
        }
        progressBar = prog

        val row = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(ctx.dp(4), ctx.dp(4), ctx.dp(4), ctx.dp(4))
        }
        backBtn = ctx.iconButton("←", pal, 38, 18f) { goBack() }
        row.addView(backBtn)

        val field = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            background = shape(pal.field, ctx.dp(20).toFloat(), pal.border, ctx.dp(1))
            setPadding(ctx.dp(10), 0, ctx.dp(2), 0)
            layoutParams = LinearLayout.LayoutParams(0, ctx.dp(40), 1f).apply { marginStart = ctx.dp(2); marginEnd = ctx.dp(4) }
        }
        lockView = ctx.text("", 12f, pal.ok, false, Gravity.CENTER).apply { setPadding(0, 0, ctx.dp(6), 0) }
        field.addView(lockView)
        val et = EditText(ctx).apply {
            background = null
            setTextColor(pal.text)
            setHintTextColor(pal.muted)
            hint = "Поиск или адрес"
            textSize = 14f
            isSingleLine = true
            setSelectAllOnFocus(true)
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_URI
            imeOptions = EditorInfo.IME_ACTION_GO or EditorInfo.IME_FLAG_NO_EXTRACT_UI
            setPadding(0, 0, 0, 0)
            layoutParams = LinearLayout.LayoutParams(0, -1, 1f)
            setOnEditorActionListener { v, _, _ -> submit(v.text.toString()); true }
            setOnKeyListener { v, code, ev ->
                if (code == KeyEvent.KEYCODE_ENTER && ev.action == KeyEvent.ACTION_UP) { submit((v as EditText).text.toString()); true } else false
            }
            setOnFocusChangeListener { v, focus ->
                settingText = true
                if (focus) {
                    setText(if (cur.home) "" else cur.url)
                    (v as EditText).selectAll()
                } else {
                    setText(displayText())
                    hideSuggestions()
                }
                settingText = false
            }
            addTextChangedListener(object : TextWatcher {
                override fun afterTextChanged(s: Editable?) { if (!settingText && hasFocus()) updateSuggestions(s?.toString().orEmpty()) }
                override fun beforeTextChanged(s: CharSequence?, st: Int, c: Int, a: Int) {}
                override fun onTextChanged(s: CharSequence?, st: Int, b: Int, c: Int) {}
            })
        }
        urlField = et
        field.addView(et)
        readerBtn = ctx.iconButton("Aa", pal, 36, 13f) { openReader() }
        reloadBtn = ctx.iconButton("⟳", pal, 36, 18f) { if (cur.loading) cur.web?.stopLoading() else reload() }
        field.addView(readerBtn)
        field.addView(reloadBtn)
        row.addView(field)

        tabsCount = ctx.text("1", 13f, pal.text, true, Gravity.CENTER).apply {
            layoutParams = LinearLayout.LayoutParams(ctx.dp(28), ctx.dp(28)).apply { marginEnd = ctx.dp(4) }
            background = shape(Color.TRANSPARENT, ctx.dp(7).toFloat(), pal.text, ctx.dp(2))
            setOnClickListener { openTabs() }
        }
        row.addView(tabsCount)
        row.addView(ctx.iconButton("⋮", pal, 38, 20f) { }.also { b -> b.setOnClickListener { showMenu(b) } })

        val div = ctx.divider(pal)
        if (prefs.addressBarBottom) { bar.addView(div); bar.addView(prog); bar.addView(row) }
        else { bar.addView(row); bar.addView(prog); bar.addView(div) }
        return bar
    }

    private fun displayText(): String = when {
        cur.home -> ""
        else -> Db.hostOf(cur.url).ifBlank { cur.url }
    }

    private fun render() {
        val et = urlField ?: return
        if (!et.hasFocus()) { settingText = true; et.setText(displayText()); settingText = false }
        val secure = cur.url.startsWith("https://")
        lockView?.apply {
            text = if (cur.home) "" else if (secure) "🔒" else "⚠"
            setTextColor(if (secure) pal.ok else pal.danger)
        }
        val p = progressBar
        if (p != null) {
            p.progress = cur.progress
            p.visibility = if (cur.loading && !cur.home) View.VISIBLE else View.INVISIBLE
        }
        reloadBtn?.text = if (cur.loading) "✕" else "⟳"
        reloadBtn?.visibility = if (cur.home) View.GONE else View.VISIBLE
        readerBtn?.visibility = if (cur.home || cur.loading) View.GONE else View.VISIBLE
        backBtn?.alpha = if (cur.home && cur.parentId == -1L) 0.35f else 1f
        updateTabsCount()
    }

    private fun showContent() {
        content.removeAllViews()
        suggestBox = null
        if (cur.home) {
            val engine = mapOf("duckduckgo" to "DuckDuckGo", "google" to "Google", "yandex" to "Яндекс", "bing" to "Bing")[prefs.searchEngine] ?: "DuckDuckGo"
            content.addView(HomeView(
                themed(), pal, db, engine, blockedTotal.get(), cur.priv,
                onOpen = { go(it) },
                onPickEngine = { openSettings() },
                onSection = { section ->
                    when (section) {
                        "Закладки" -> openBookmarks(false)
                        "История" -> openBookmarks(true)
                        "Загрузки" -> openDownloads()
                        else -> openSettings()
                    }
                },
                onChanged = { showContent() },
            ).view, matchMatch())
        } else {
            val w = ensureWeb(cur)
            (w.parent as? ViewGroup)?.removeView(w)
            content.addView(w, matchMatch())
            w.onResume()
        }
    }

    private fun toast(s: String) = Toast.makeText(this, s, Toast.LENGTH_SHORT).show()

    private fun hideKeyboard() {
        urlField?.clearFocus()
        (getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager).hideSoftInputFromWindow(root.windowToken, 0)
        content.requestFocus()
    }

    // ───────────────────────── подсказки ─────────────────────────

    private fun updateSuggestions(q: String) {
        if (q.isBlank()) { hideSuggestions(); return }
        val results = Core.suggest(q, db.suggestEntries(), 6)
        val box = suggestBox ?: LinearLayout(themed()).apply {
            orientation = LinearLayout.VERTICAL
            setBackgroundColor(pal.bar)
            elevation = dp(6).toFloat()
            isClickable = true
        }.also {
            suggestBox = it
            content.addView(it, FrameLayout.LayoutParams(-1, -2, if (prefs.addressBarBottom) Gravity.BOTTOM else Gravity.TOP))
        }
        box.removeAllViews()
        val ctx = box.context
        val rows = ArrayList<Pair<String, String>>()
        rows += "Искать: $q" to q
        rows += results
        rows.forEachIndexed { i, (title, url) ->
            val row = LinearLayout(ctx).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(ctx.dp(16), ctx.dp(9), ctx.dp(16), ctx.dp(9))
                background = ripple(pal.ripple, null)
                addView(ctx.text(title.ifBlank { url }, 14f, pal.text).apply { maxLines = 1; ellipsize = android.text.TextUtils.TruncateAt.END })
                if (i > 0) addView(ctx.text(url, 11f, pal.muted).apply { maxLines = 1; ellipsize = android.text.TextUtils.TruncateAt.END; setPadding(0, ctx.dp(2), 0, 0) })
                setOnClickListener { submit(if (i == 0) q else url) }
            }
            box.addView(row)
        }
    }

    private fun hideSuggestions() {
        suggestBox?.let { content.removeView(it) }
        suggestBox = null
    }

    // ───────────────────────── навигация ─────────────────────────

    private fun submit(input: String) {
        hideSuggestions()
        hideKeyboard()
        go(input.trim())
    }

    private fun go(input: String) {
        val r = Core.resolve(input, prefs.searchEngine)
        if (r.kind == InputKind.INTERNAL) goHome(cur) else load(cur, r.url)
    }

    private fun load(t: Tab, url: String) {
        t.url = url
        t.pageUrl = url
        t.home = false
        t.title = ""
        t.progress = 0
        t.loading = true
        t.blocked.set(0)
        val w = ensureWeb(t, loadNow = false)
        if (t === cur) { showContent(); render() }
        w.loadUrl(url)
    }

    private fun goHome(t: Tab) {
        t.web?.stopLoading()
        t.home = true
        t.url = START_URL
        t.title = ""
        t.loading = false
        t.progress = 0
        if (t === cur) { closeFind(); showContent(); render() }
    }

    private fun reload() {
        val w = cur.web ?: return
        w.reload()
    }

    private fun goBack() {
        when {
            panels.isNotEmpty() -> closePanel()
            readerView != null -> closeReader()
            findBar != null -> closeFind()
            customView != null -> hideCustomView()
            suggestBox != null -> { hideSuggestions(); hideKeyboard() }
            !cur.home && cur.web?.canGoBack() == true -> cur.web?.goBack()
            !cur.home -> {
                if (tabs.any { it.id == cur.parentId }) closeTab(cur) else goHome(cur)
            }
            tabs.any { it.id == cur.parentId } -> closeTab(cur)
            else -> finish()
        }
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() = goBack()

    // ───────────────────────── WebView ─────────────────────────

    @SuppressLint("SetJavaScriptEnabled")
    private fun ensureWeb(t: Tab, loadNow: Boolean = true): WebView {
        t.web?.let { return it }
        val w = WebView(themed(Pal(prefs.theme, t.priv)))
        w.setBackgroundColor(Pal(prefs.theme, t.priv).bg)
        t.web = w
        configure(w, t)
        w.webViewClient = client(t)
        w.webChromeClient = chrome(t)
        w.setDownloadListener(DownloadListener { url, ua, cd, mime, _ -> startDownload(url, ua, cd, mime, t) })
        w.setOnLongClickListener { onLongPress(w, t) }
        if (loadNow && !t.home && t.url != START_URL) w.loadUrl(t.url)
        return w
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun configure(w: WebView, t: Tab) {
        val s = w.settings
        s.javaScriptEnabled = prefs.javaScript
        s.domStorageEnabled = !t.priv
        s.databaseEnabled = !t.priv
        s.cacheMode = if (t.priv) WebSettings.LOAD_NO_CACHE else WebSettings.LOAD_DEFAULT
        s.saveFormData = !t.priv
        s.allowFileAccess = false
        s.allowContentAccess = false
        s.setGeolocationEnabled(false)
        s.mediaPlaybackRequiresUserGesture = true
        s.javaScriptCanOpenWindowsAutomatically = false
        s.setSupportMultipleWindows(false)
        s.mixedContentMode = WebSettings.MIXED_CONTENT_COMPATIBILITY_MODE
        s.blockNetworkImage = prefs.blockImages
        s.loadsImagesAutomatically = !prefs.blockImages
        s.textZoom = prefs.textZoom
        s.builtInZoomControls = true
        s.displayZoomControls = false
        if (prefs.desktopMode) {
            s.userAgentString = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
            s.useWideViewPort = true
            s.loadWithOverviewMode = true
        } else {
            s.userAgentString = defaultUa.ifEmpty { null }
            s.useWideViewPort = true
            s.loadWithOverviewMode = false
        }
        if (Build.VERSION.SDK_INT >= 29) {
            @Suppress("DEPRECATION")
            s.forceDark = if (prefs.darkPages) WebSettings.FORCE_DARK_ON else WebSettings.FORCE_DARK_AUTO
        }
        CookieManager.getInstance().setAcceptCookie(true)
        CookieManager.getInstance().setAcceptThirdPartyCookies(w, !prefs.blockThirdPartyCookies)
        w.isHorizontalScrollBarEnabled = false
    }

    private fun applySettings() {
        Core.configureBlocker(prefs.blockAds, prefs.blockTrackers)
        tabs.forEach { t -> t.web?.let { configure(it, t) } }
        cur.web?.reload()
    }

    private fun client(t: Tab) = object : WebViewClient() {
        override fun shouldInterceptRequest(view: WebView, request: WebResourceRequest): WebResourceResponse? {
            if (request.isForMainFrame || (!prefs.blockAds && !prefs.blockTrackers)) return null
            val verdict = Core.check(request.url.toString(), t.pageUrl)
            if (verdict == Verdict.ALLOW) return null
            t.blocked.incrementAndGet()
            blockedTotal.incrementAndGet()
            return WebResourceResponse("text/plain", "utf-8", ByteArrayInputStream(ByteArray(0)))
        }

        override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean = handleScheme(request.url.toString(), t)

        override fun onPageStarted(view: WebView, url: String, favicon: Bitmap?) {
            t.url = url
            t.pageUrl = url
            t.home = false
            t.loading = true
            t.blocked.set(0)
            if (t === cur) render()
        }

        override fun onPageFinished(view: WebView, url: String) {
            t.loading = false
            t.progress = 100
            val title = view.title.orEmpty()
            if (title.isNotBlank() && title != url) t.title = title
            if (url.startsWith("http") && !t.priv) {
                db.visit(t.title, url)
            }
            if (t === cur) render()
        }

        override fun doUpdateVisitedHistory(view: WebView, url: String, isReload: Boolean) {
            if (url.startsWith("http")) { t.url = url; if (t === cur) render() }
        }

        @Deprecated("Deprecated in Java")
        override fun onReceivedError(view: WebView, errorCode: Int, description: String?, failingUrl: String?) {
            if (Build.VERSION.SDK_INT < 23 && failingUrl != null && failingUrl == t.url) showError(view, t, description ?: "Ошибка загрузки", failingUrl)
        }

        override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) {
            if (Build.VERSION.SDK_INT >= 23 && request.isForMainFrame) showError(view, t, error.description?.toString() ?: "Ошибка загрузки", request.url.toString())
        }

        override fun onReceivedSslError(view: WebView, handler: SslErrorHandler, error: SslError) {
            handler.cancel()
            showError(view, t, "Сертификат сайта не прошёл проверку. Соединение небезопасно.", error.url ?: t.url)
        }
    }

    private fun showError(view: WebView, t: Tab, details: String, url: String) {
        t.loading = false
        view.loadDataWithBaseURL(url, ReaderPage.errorPage(Pal(prefs.theme, t.priv), "Не удалось открыть страницу", details, url), "text/html", "utf-8", url)
        if (t === cur) render()
    }

    private fun chrome(t: Tab) = object : WebChromeClient() {
        override fun onProgressChanged(view: WebView, newProgress: Int) {
            t.progress = newProgress
            t.loading = newProgress < 100
            if (t === cur) render()
        }

        override fun onReceivedTitle(view: WebView, title: String?) {
            if (!title.isNullOrBlank()) {
                t.title = title
                if (!t.priv && t.url.startsWith("http")) db.updateHistoryTitle(t.url, title)
            }
        }

        override fun onShowFileChooser(view: WebView, cb: ValueCallback<Array<Uri>>, params: FileChooserParams): Boolean {
            fileCallback?.onReceiveValue(null)
            fileCallback = cb
            return try {
                startActivityForResult(params.createIntent(), REQ_FILE)
                true
            } catch (_: Exception) {
                fileCallback = null
                false
            }
        }

        override fun onShowCustomView(view: View, callback: CustomViewCallback) {
            if (customView != null) { callback.onCustomViewHidden(); return }
            customView = view
            customCallback = callback
            view.setBackgroundColor(Color.BLACK)
            root.addView(view, matchMatch())
            window.decorView.systemUiVisibility = View.SYSTEM_UI_FLAG_FULLSCREEN or View.SYSTEM_UI_FLAG_HIDE_NAVIGATION or View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
        }

        override fun onHideCustomView() = hideCustomView()
    }

    private fun hideCustomView() {
        customView?.let { root.removeView(it) }
        customView = null
        customCallback?.onCustomViewHidden()
        customCallback = null
        applyChrome()
    }

    @Deprecated("Deprecated in Java")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        if (requestCode == REQ_FILE) {
            fileCallback?.onReceiveValue(WebChromeClient.FileChooserParams.parseResult(resultCode, data))
            fileCallback = null
        } else {
            @Suppress("DEPRECATION") super.onActivityResult(requestCode, resultCode, data)
        }
    }

    /** true — навигацию обработали мы (WebView её не выполняет). */
    private fun handleScheme(url: String, t: Tab): Boolean {
        val uri = Uri.parse(url)
        when (uri.scheme?.lowercase()) {
            "http", "https", "data", "blob", "about" -> return false
            "tyt" -> { goHome(t); return true }
            "mailto", "tel", "sms", "geo", "market" -> { startExternal(Intent(Intent.ACTION_VIEW, uri)); return true }
            "intent" -> {
                try {
                    val i = Intent.parseUri(url, Intent.URI_INTENT_SCHEME)
                    // Защита от подмены компонентов: страница не может адресовать произвольное приложение/сервис.
                    i.addCategory(Intent.CATEGORY_BROWSABLE)
                    i.component = null
                    i.selector = null
                    if (i.resolveActivity(packageManager) != null) startExternal(i)
                    else i.getStringExtra("browser_fallback_url")?.takeIf { it.startsWith("http") }?.let { load(t, it) }
                } catch (_: Exception) {
                }
                return true
            }
            else -> return true // javascript:, file:, content: и прочее не открываем
        }
    }

    private fun startExternal(i: Intent) {
        try {
            startActivity(i)
        } catch (_: Exception) {
            toast("Нет приложения для открытия ссылки")
        }
    }

    // ───────────────────────── долгое нажатие ─────────────────────────

    private fun onLongPress(w: WebView, t: Tab): Boolean {
        val r = w.hitTestResult
        val extra = r.extra ?: return false
        return when (r.type) {
            WebView.HitTestResult.SRC_ANCHOR_TYPE, WebView.HitTestResult.SRC_IMAGE_ANCHOR_TYPE -> {
                val items = arrayOf("Открыть в новой вкладке", "Открыть в приватной вкладке", "Копировать ссылку", "Поделиться ссылкой")
                AlertDialog.Builder(themed()).setTitle(Db.hostOf(extra)).setItems(items) { _, which ->
                    when (which) {
                        0 -> newTab(extra, priv = t.priv, foreground = false, parent = t)
                        1 -> newTab(extra, priv = true, foreground = true, parent = t)
                        2 -> copy(extra)
                        3 -> share(extra)
                    }
                }.show()
                true
            }
            WebView.HitTestResult.IMAGE_TYPE -> {
                AlertDialog.Builder(themed()).setItems(arrayOf("Скачать изображение", "Копировать адрес изображения")) { _, which ->
                    if (which == 0) startDownload(extra, w.settings.userAgentString, null, null, t) else copy(extra)
                }.show()
                true
            }
            else -> false
        }
    }

    private fun shareable(url: String) = if (prefs.stripTracking) Core.cleanUrl(url) else url

    private fun copy(url: String) {
        val u = shareable(url)
        (getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager).setPrimaryClip(ClipData.newPlainText("url", u))
        toast(if (u != url) "Ссылка скопирована и очищена от трекинга" else "Ссылка скопирована")
    }

    private fun share(url: String) {
        startExternal(Intent.createChooser(Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, shareable(url)), "Поделиться"))
    }

    // ───────────────────────── загрузки ─────────────────────────

    private fun startDownload(url: String, ua: String?, cd: String?, mime: String?, t: Tab) {
        if (!(url.startsWith("http://") || url.startsWith("https://"))) { toast("Такую ссылку скачать нельзя"); return }
        val doIt = {
            try {
                val name = URLUtil.guessFileName(url, cd, mime)
                val req = DownloadManager.Request(Uri.parse(url))
                    .setNotificationVisibility(DownloadManager.Request.VISIBILITY_VISIBLE_NOTIFY_COMPLETED)
                    .setDestinationInExternalPublicDir(Environment.DIRECTORY_DOWNLOADS, name)
                if (!mime.isNullOrBlank()) req.setMimeType(mime)
                if (!ua.isNullOrBlank()) req.addRequestHeader("User-Agent", ua)
                CookieManager.getInstance().getCookie(url)?.let { req.addRequestHeader("Cookie", it) }
                val id = (getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager).enqueue(req)
                db.addDownload(id, name, url)
                toast("Загрузка: $name")
            } catch (e: Exception) {
                toast("Не удалось начать загрузку")
            }
        }
        if (Build.VERSION.SDK_INT < 29 && checkSelfPermission(Manifest.permission.WRITE_EXTERNAL_STORAGE) != PackageManager.PERMISSION_GRANTED) {
            pendingDownload = doIt
            requestPermissions(arrayOf(Manifest.permission.WRITE_EXTERNAL_STORAGE), REQ_STORAGE)
        } else doIt()
    }

    override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == REQ_STORAGE) {
            val job = pendingDownload
            pendingDownload = null
            if (grantResults.firstOrNull() == PackageManager.PERMISSION_GRANTED) job?.invoke() else toast("Нужно разрешение на запись, чтобы сохранить файл")
        }
    }

    // ───────────────────────── меню ─────────────────────────

    private fun showMenu(anchor: View) {
        val pm = PopupMenu(themed(), anchor)
        val m = pm.menu
        var order = 0
        fun item(id: Int, title: String, checkable: Boolean = false, checked: Boolean = false, enabled: Boolean = true) {
            m.add(0, id, order++, title).apply { isCheckable = checkable; isChecked = checked; isEnabled = enabled }
        }
        if (!cur.home) item(100, "Заблокировано на странице: ${cur.blocked.get()}", enabled = false)
        item(1, "Новая вкладка")
        item(2, "Новая приватная вкладка")
        if (!cur.home) {
            item(3, "Вперёд", enabled = cur.web?.canGoForward() == true)
            item(4, "Обновить")
            val bookmarked = db.isBookmarked(cur.url)
            item(5, if (bookmarked) "Убрать из закладок" else "В закладки")
            item(6, "Режим чтения")
            item(7, "Найти на странице")
            item(8, "Копировать ссылку")
            item(9, "Поделиться")
        }
        item(10, "Главная страница", enabled = !cur.home)
        item(11, "Версия для ПК", checkable = true, checked = prefs.desktopMode)
        item(12, "Блокировка рекламы", checkable = true, checked = prefs.blockAds && prefs.blockTrackers)
        item(13, "Без изображений", checkable = true, checked = prefs.blockImages)
        item(14, "Закладки")
        item(15, "История")
        item(16, "Загрузки")
        item(17, "Настройки")
        item(18, "Очистить данные и выйти")
        pm.setOnMenuItemClickListener { mi ->
            when (mi.itemId) {
                1 -> newTab(null, priv = false, foreground = true)
                2 -> newTab(null, priv = true, foreground = true)
                3 -> cur.web?.goForward()
                4 -> reload()
                5 -> {
                    if (db.isBookmarked(cur.url)) { db.removeBookmarkByUrl(cur.url); toast("Закладка удалена") }
                    else { db.addBookmark(cur.title, cur.url); toast("Добавлено в закладки") }
                }
                6 -> openReader()
                7 -> openFind()
                8 -> copy(cur.url)
                9 -> share(cur.url)
                10 -> goHome(cur)
                11 -> { prefs.desktopMode = !prefs.desktopMode; applySettings() }
                12 -> { val on = !(prefs.blockAds && prefs.blockTrackers); prefs.blockAds = on; prefs.blockTrackers = on; applySettings() }
                13 -> { prefs.blockImages = !prefs.blockImages; applySettings() }
                14 -> openBookmarks(false)
                15 -> openBookmarks(true)
                16 -> openDownloads()
                17 -> openSettings()
                18 -> { clearAllData(); finishAndRemoveTask() }
            }
            true
        }
        pm.show()
    }

    // ───────────────────────── панели ─────────────────────────

    private fun openPanel(v: View) {
        hideKeyboard()
        hideSuggestions()
        overlay.addView(v, matchMatch())
        panels += v
    }

    private fun closePanel() {
        val v = panels.removeLastOrNull() ?: return
        overlay.removeView(v)
        if (cur.home) showContent()
    }

    private fun openBookmarks(history: Boolean): Unit = openPanel(
        BookmarksHistoryPanel(themed(), pal, db, history, { closePanel(); go(it) }, { newTab(it, priv = cur.priv, foreground = false, parent = cur) }, { closePanel() }).root
    )

    private fun openDownloads(): Unit = openPanel(DownloadsPanel(themed(), pal, db) { closePanel() }.root)

    private fun openTabs(): Unit = openPanel(
        TabsPanel(
            themed(), pal,
            tabs = { tabs.map { TabsPanel.TabInfo(it.id, it.displayTitle, if (it.home) "Быстрый доступ" else it.url, it.priv) } },
            activeId = cur.id,
            select = { id -> tabs.firstOrNull { it.id == id }?.let { closePanel(); select(it) } },
            closeTab = { id -> tabs.firstOrNull { it.id == id }?.let { closeTab(it) } },
            newTab = { priv -> closePanel(); newTab(null, priv, foreground = true) },
            closeAll = { priv -> closePanel(); closeAllTabs(priv); openTabs() },
            close = { closePanel() },
        ).root
    )

    private fun openSettings() {
        val themeBefore = prefs.theme
        val barBefore = prefs.addressBarBottom
        openPanel(
            SettingsPanel(
                themed(), pal, prefs,
                onChanged = {
                    applySettings()
                    if (prefs.theme != themeBefore || prefs.addressBarBottom != barBefore) {
                        closePanel(); applyChrome(); openSettings()
                    }
                },
                onClearData = { clearAllData() },
                onSetDefault = {
                    startExternal(Intent(android.provider.Settings.ACTION_MANAGE_DEFAULT_APPS_SETTINGS))
                },
                close = { closePanel() },
            ).root
        )
    }

    // ───────────────────────── режим чтения ─────────────────────────

    private fun openReader() {
        val w = cur.web ?: return
        if (!prefs.javaScript) { toast("Режим чтения требует включённого JavaScript"); return }
        toast("Готовлю режим чтения…")
        val base = cur.url
        w.evaluateJavascript("(function(){return document.documentElement.outerHTML})()") { raw ->
            thread(name = "reader") {
                val html = try { JSONTokener(raw).nextValue() as? String } catch (_: Exception) { null }
                val article = if (html.isNullOrEmpty()) null else Core.extractReadable(html, base)
                runOnUiThread {
                    if (isFinishing) return@runOnUiThread
                    if (article == null) toast("На этой странице нет статьи для чтения") else showReader(article)
                }
            }
        }
    }

    private fun showReader(article: JSONObject) {
        closeReader()
        readerArticle = article
        val ctx = themed()
        val rp = readerPal()
        val col = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL; setBackgroundColor(rp.bg); isClickable = true }
        val web = WebView(ctx).apply {
            settings.javaScriptEnabled = false
            settings.allowFileAccess = false
            settings.allowContentAccess = false
            setBackgroundColor(rp.bg)
        }
        fun refresh() {
            val th = readerTheme()
            val p = Pal(if (th >= 2) 2 else 0, false)
            web.setBackgroundColor(when (th) { 0 -> 0xFFFAF9F5.toInt(); 1 -> 0xFFF4ECD8.toInt(); 2 -> 0xFF1E1E1E.toInt(); else -> Color.BLACK })
            web.loadDataWithBaseURL(null, ReaderPage.build(article, prefs.readerFontPx, th, prefs.readerSerif), "text/html", "utf-8", null)
            col.setBackgroundColor(p.bg)
        }
        val bar = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setBackgroundColor(rp.bar)
            setPadding(ctx.dp(4), ctx.dp(4), ctx.dp(8), ctx.dp(4))
        }
        bar.addView(ctx.iconButton("✕", rp, 40, 16f) { closeReader() })
        bar.addView(ctx.text("Режим чтения", 15f, rp.text, true).apply { layoutParams = LinearLayout.LayoutParams(0, -2, 1f) })
        bar.addView(ctx.iconButton("A−", rp, 40, 13f) { prefs.readerFontPx = (prefs.readerFontPx - 2).coerceAtLeast(13); refresh() })
        bar.addView(ctx.iconButton("A+", rp, 40, 13f) { prefs.readerFontPx = (prefs.readerFontPx + 2).coerceAtMost(30); refresh() })
        bar.addView(ctx.iconButton("◐", rp, 40, 18f) { prefs.readerTheme = (readerTheme() + 1) % 4; refresh() })
        bar.addView(ctx.iconButton("Aa", rp, 40, 13f) { prefs.readerSerif = !prefs.readerSerif; refresh() })
        col.addView(bar, LinearLayout.LayoutParams(-1, -2))
        col.addView(web, LinearLayout.LayoutParams(-1, 0, 1f))
        refresh()
        overlay.addView(col, matchMatch())
        readerView = col
    }

    private fun readerTheme(): Int = prefs.readerTheme.let { if (it in 0..3) it else ReaderPage.defaultTheme(prefs.theme) }
    private fun readerPal(): Pal = Pal(if (readerTheme() >= 2) 2 else 0, false)

    private fun closeReader() {
        readerView?.let { v ->
            (v as? ViewGroup)?.let { g -> for (i in 0 until g.childCount) (g.getChildAt(i) as? WebView)?.destroy() }
            overlay.removeView(v)
        }
        readerView = null
        readerArticle = null
    }

    // ───────────────────────── поиск по странице ─────────────────────────

    private fun openFind() {
        val w = cur.web ?: return
        closeFind()
        val ctx = themed()
        val bar = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setBackgroundColor(pal.bar)
            elevation = ctx.dp(8).toFloat()
            setPadding(ctx.dp(8), ctx.dp(4), ctx.dp(4), ctx.dp(4))
            isClickable = true
        }
        val count = ctx.text("", 12f, pal.muted, false, Gravity.CENTER).apply { setPadding(ctx.dp(8), 0, ctx.dp(4), 0) }
        val et = EditText(ctx).apply {
            hint = "Найти на странице"
            setHintTextColor(pal.muted); setTextColor(pal.text); textSize = 14f
            isSingleLine = true
            imeOptions = EditorInfo.IME_ACTION_SEARCH or EditorInfo.IME_FLAG_NO_EXTRACT_UI
            layoutParams = LinearLayout.LayoutParams(0, -2, 1f)
            addTextChangedListener(object : TextWatcher {
                override fun afterTextChanged(s: Editable?) {
                    val q = s?.toString().orEmpty()
                    if (q.isEmpty()) { w.clearMatches(); count.text = "" } else w.findAllAsync(q)
                }
                override fun beforeTextChanged(s: CharSequence?, st: Int, c: Int, a: Int) {}
                override fun onTextChanged(s: CharSequence?, st: Int, b: Int, c: Int) {}
            })
            setOnEditorActionListener { _, _, _ -> w.findNext(true); true }
        }
        w.setFindListener { active, total, done -> if (done) count.text = if (total == 0) "0" else "${active + 1}/$total" }
        bar.addView(et); bar.addView(count)
        bar.addView(ctx.iconButton("▲", pal, 36, 14f) { w.findNext(false) })
        bar.addView(ctx.iconButton("▼", pal, 36, 14f) { w.findNext(true) })
        bar.addView(ctx.iconButton("✕", pal, 36, 14f) { closeFind() })
        val lp = FrameLayout.LayoutParams(-1, -2, if (prefs.addressBarBottom) Gravity.TOP else Gravity.BOTTOM)
        content.addView(bar, lp)
        findBar = bar
        et.requestFocus()
        (getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager).showSoftInput(et, 0)
    }

    private fun closeFind() {
        findBar?.let { content.removeView(it) }
        findBar = null
        cur.web?.clearMatches()
    }

    // ───────────────────────── данные ─────────────────────────

    private fun clearAllData() {
        db.clearHistory()
        db.clearDownloads()
        tabs.forEach { t -> t.web?.apply { clearCache(true); clearHistory(); clearFormData() } }
        try {
            CookieManager.getInstance().removeAllCookies(null)
            CookieManager.getInstance().flush()
            WebStorage.getInstance().deleteAllData()
        } catch (_: Throwable) {
        }
    }

    /** Свои правила: filters/ads.txt и filters/trackers.txt во внутренней папке приложения (необязательно). */
    private fun loadUserRules() {
        val dir = java.io.File(filesDir, "filters")
        for ((name, trackers) in listOf("ads.txt" to false, "trackers.txt" to true)) {
            val f = java.io.File(dir, name)
            if (f.isFile && f.length() < 8_000_000) {
                try { Core.addRules(trackers, f.readText()) } catch (_: Exception) {}
            }
        }
    }

    companion object {
        private const val REQ_FILE = 41
        private const val REQ_STORAGE = 42
    }
}
