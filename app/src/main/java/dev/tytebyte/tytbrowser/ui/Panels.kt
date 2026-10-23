package dev.tytebyte.tytbrowser.ui

import android.app.AlertDialog
import android.app.DownloadManager
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.net.Uri
import android.text.Editable
import android.text.InputType
import android.text.TextUtils
import android.text.TextWatcher
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.Toast
import dev.tytebyte.tytbrowser.BuildConfig
import dev.tytebyte.tytbrowser.GeoData
import dev.tytebyte.tytbrowser.core.Core
import dev.tytebyte.tytbrowser.data.Db
import dev.tytebyte.tytbrowser.data.Prefs
import dev.tytebyte.tytbrowser.data.RouteAction
import dev.tytebyte.tytbrowser.data.RouteRule
import dev.tytebyte.tytbrowser.data.RouteRules

/** Полноэкранный раздел поверх браузера: шапка со стрелкой «назад», заголовком и действиями. */
open class Panel(protected val ctx: Context, protected val pal: Pal, title: String, private val close: () -> Unit) {
    val root = LinearLayout(ctx).apply {
        orientation = LinearLayout.VERTICAL
        setBackgroundColor(pal.bg)
        isClickable = true // не пропускаем касания к странице под панелью
    }
    protected val header = LinearLayout(ctx).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        setBackgroundColor(pal.bar)
        setPadding(ctx.dp(4), ctx.dp(4), ctx.dp(8), ctx.dp(4))
    }
    protected val body = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL }

    init {
        header.addView(ctx.iconButton("←", pal) { close() })
        header.addView(ctx.text(title, 18f, pal.text, true).apply { layoutParams = weighted().apply { marginStart = ctx.dp(4) } })
        root.addView(header, matchWrap())
        root.addView(ctx.divider(pal))
        root.addView(body, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
    }

    protected fun action(label: String, onClick: () -> Unit) {
        header.addView(ctx.text(label, 13f, pal.accent, true, Gravity.CENTER).apply {
            setPadding(ctx.dp(12), ctx.dp(8), ctx.dp(12), ctx.dp(8))
            setOnClickListener { onClick() }
        })
    }

    protected fun scroll(content: View): ScrollView = ScrollView(ctx).apply {
        isFillViewport = true
        addView(content, ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT))
    }

    protected fun empty(text: String): View = ctx.text(text, 14f, pal.muted, false, Gravity.CENTER).apply {
        setPadding(ctx.dp(24), ctx.dp(48), ctx.dp(24), ctx.dp(48))
        layoutParams = matchWrap()
    }

    protected fun row(title: String, subtitle: String, onClick: () -> Unit, onDelete: (() -> Unit)?, onLong: (() -> Unit)? = null): View =
        LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(ctx.dp(16), ctx.dp(8), ctx.dp(4), ctx.dp(8))
            background = ripple(pal.ripple, null)
            val col = LinearLayout(ctx).apply {
                orientation = LinearLayout.VERTICAL
                layoutParams = weighted()
                addView(ctx.text(title.ifBlank { subtitle }, 14f, pal.text).apply {
                    maxLines = 1; ellipsize = TextUtils.TruncateAt.END
                })
                addView(ctx.text(subtitle, 11f, pal.muted).apply {
                    maxLines = 1; ellipsize = TextUtils.TruncateAt.END
                    setPadding(0, ctx.dp(3), 0, 0)
                })
            }
            addView(col)
            if (onDelete != null) addView(ctx.iconButton("✕", pal, 40, 15f) { onDelete() })
            setOnClickListener { onClick() }
            if (onLong != null) setOnLongClickListener { onLong(); true }
        }
}

/** Закладки и история с поиском. */
class BookmarksHistoryPanel(
    ctx: Context, pal: Pal, private val db: Db, startOnHistory: Boolean,
    private val open: (String) -> Unit, private val openNewTab: (String) -> Unit, close: () -> Unit,
) : Panel(ctx, pal, if (startOnHistory) "История" else "Закладки", close) {

    private var history = startOnHistory
    private var filter = ""
    private val list = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL }
    private val tabBookmarks = ctx.text("Закладки", 13f, pal.text, true, Gravity.CENTER)
    private val tabHistory = ctx.text("История", 13f, pal.text, true, Gravity.CENTER)

    init {
        action("Очистить") { if (history) confirmClear() }
        val tabs = LinearLayout(ctx).apply { orientation = LinearLayout.HORIZONTAL }
        listOf(tabBookmarks to false, tabHistory to true).forEach { (v, h) ->
            v.setPadding(0, ctx.dp(12), 0, ctx.dp(12))
            v.layoutParams = weighted()
            v.setOnClickListener { history = h; render() }
            tabs.addView(v)
        }
        body.addView(tabs, matchWrap())
        body.addView(EditText(ctx).apply {
            hint = "Фильтр по названию или ссылке"
            setHintTextColor(pal.muted); setTextColor(pal.text); textSize = 14f
            isSingleLine = true
            background = shape(pal.field, ctx.dp(12).toFloat())
            setPadding(ctx.dp(14), ctx.dp(10), ctx.dp(14), ctx.dp(10))
            layoutParams = matchWrap().apply { setMargins(ctx.dp(12), ctx.dp(4), ctx.dp(12), ctx.dp(8)) }
            addTextChangedListener(object : TextWatcher {
                override fun afterTextChanged(s: Editable?) { filter = s?.toString().orEmpty(); renderList() }
                override fun beforeTextChanged(s: CharSequence?, st: Int, c: Int, a: Int) {}
                override fun onTextChanged(s: CharSequence?, st: Int, b: Int, c: Int) {}
            })
        })
        body.addView(scroll(list), LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
        render()
    }

    private fun render() {
        tabBookmarks.setTextColor(if (!history) pal.accent else pal.muted)
        tabHistory.setTextColor(if (history) pal.accent else pal.muted)
        renderList()
    }

    private fun renderList() {
        list.removeAllViews()
        val f = filter.trim().lowercase()
        if (!history) {
            val items = db.bookmarks().filter { f.isEmpty() || it.title.lowercase().contains(f) || it.url.lowercase().contains(f) }
            if (items.isEmpty()) list.addView(empty("Закладок нет"))
            items.forEach { b ->
                list.addView(row(b.title, b.url, { open(b.url) }, { db.removeBookmark(b.id); renderList() }, { openNewTab(b.url) }))
            }
        } else {
            val items = db.history(filter)
            if (items.isEmpty()) list.addView(empty("История пуста"))
            items.forEach { h ->
                list.addView(row(h.title, "${Db.whenText(h.visitedAt)} · ${h.url}", { open(h.url) }, { db.deleteHistory(h.id); renderList() }, { openNewTab(h.url) }))
            }
        }
    }

    private fun confirmClear() {
        AlertDialog.Builder(ctx).setMessage("Очистить всю историю посещений?")
            .setNegativeButton("Отмена", null)
            .setPositiveButton("Очистить") { _, _ -> db.clearHistory(); renderList() }.show()
    }
}

/** Загрузки (через системный DownloadManager). */
class DownloadsPanel(ctx: Context, pal: Pal, private val db: Db, close: () -> Unit) : Panel(ctx, pal, "Загрузки", close) {
    private val dm = ctx.getSystemService(Context.DOWNLOAD_SERVICE) as DownloadManager
    private val list = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL }
    private val handler = android.os.Handler(android.os.Looper.getMainLooper())
    private val tick = object : Runnable {
        override fun run() { render(); if (root.isAttachedToWindow) handler.postDelayed(this, 1500) }
    }

    init {
        action("Очистить") {
            db.clearDownloads(); render()
        }
        body.addView(scroll(list), LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
        root.addOnAttachStateChangeListener(object : View.OnAttachStateChangeListener {
            override fun onViewAttachedToWindow(v: View) { handler.post(tick) }
            override fun onViewDetachedFromWindow(v: View) { handler.removeCallbacks(tick) }
        })
        render()
    }

    private fun humanSize(b: Long): String = when {
        b >= 1L shl 30 -> String.format("%.1f ГБ", b / 1073741824.0)
        b >= 1L shl 20 -> String.format("%.1f МБ", b / 1048576.0)
        b >= 1L shl 10 -> String.format("%.0f КБ", b / 1024.0)
        else -> "$b Б"
    }

    private fun render() {
        list.removeAllViews()
        val items = db.downloads()
        if (items.isEmpty()) { list.addView(empty("Загрузок нет")); return }
        for (d in items) {
            var status = DownloadManager.STATUS_FAILED
            var done = 0L
            var total = -1L
            var exists = false
            try {
                dm.query(DownloadManager.Query().setFilterById(d.dmId))?.use { c ->
                    if (c.moveToFirst()) {
                        exists = true
                        status = c.getInt(c.getColumnIndexOrThrow(DownloadManager.COLUMN_STATUS))
                        done = c.getLong(c.getColumnIndexOrThrow(DownloadManager.COLUMN_BYTES_DOWNLOADED_SO_FAR))
                        total = c.getLong(c.getColumnIndexOrThrow(DownloadManager.COLUMN_TOTAL_SIZE_BYTES))
                    }
                }
            } catch (_: Exception) {
            }
            val sub = when {
                !exists -> "Файл удалён"
                status == DownloadManager.STATUS_SUCCESSFUL -> "Готово · ${humanSize(total.coerceAtLeast(done))} · ${Db.whenText(d.at)}"
                status == DownloadManager.STATUS_FAILED -> "Ошибка загрузки"
                status == DownloadManager.STATUS_PAUSED -> "Пауза · ${humanSize(done)}"
                total > 0 -> "Загрузка ${(done * 100 / total)}% · ${humanSize(done)} из ${humanSize(total)}"
                else -> "Загрузка… ${humanSize(done)}"
            }
            list.addView(row(d.name, sub, {
                if (exists && status == DownloadManager.STATUS_SUCCESSFUL) openFile(d.dmId)
            }, {
                if (exists) try { dm.remove(d.dmId) } catch (_: Exception) {}
                db.removeDownload(d.dmId); render()
            }))
        }
    }

    private fun openFile(id: Long) {
        try {
            val uri = dm.getUriForDownloadedFile(id) ?: return
            val mime = dm.getMimeTypeForDownloadedFile(id) ?: "*/*"
            ctx.startActivity(Intent(Intent.ACTION_VIEW).setDataAndType(uri, mime).addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK))
        } catch (_: Exception) {
            Toast.makeText(ctx, "Нет приложения для открытия файла", Toast.LENGTH_SHORT).show()
        }
    }
}

/** Переключатель вкладок. */
class TabsPanel(
    ctx: Context, pal: Pal,
    private val tabs: () -> List<TabInfo>,
    private val activeId: Long,
    private val select: (Long) -> Unit,
    private val closeTab: (Long) -> Unit,
    private val newTab: (Boolean) -> Unit,
    private val closeAll: (Boolean) -> Unit,
    close: () -> Unit,
) : Panel(ctx, pal, "Вкладки", close) {

    data class TabInfo(val id: Long, val title: String, val url: String, val priv: Boolean)

    private var showPrivate = tabs().firstOrNull { it.id == activeId }?.priv ?: false
    private val list = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL }
    private val segRegular = ctx.text("", 13f, pal.text, true, Gravity.CENTER)
    private val segPrivate = ctx.text("", 13f, pal.text, true, Gravity.CENTER)

    init {
        val seg = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            layoutParams = matchWrap().apply { setMargins(ctx.dp(12), ctx.dp(10), ctx.dp(12), ctx.dp(6)) }
            background = shape(pal.field, ctx.dp(12).toFloat())
            setPadding(ctx.dp(3), ctx.dp(3), ctx.dp(3), ctx.dp(3))
        }
        listOf(segRegular to false, segPrivate to true).forEach { (v, p) ->
            v.setPadding(0, ctx.dp(9), 0, ctx.dp(9)); v.layoutParams = weighted()
            v.setOnClickListener { showPrivate = p; render() }
            seg.addView(v)
        }
        body.addView(seg)
        body.addView(scroll(list), LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
        val bottom = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setBackgroundColor(pal.bar)
            setPadding(ctx.dp(8), ctx.dp(8), ctx.dp(12), ctx.dp(8))
        }
        bottom.addView(ctx.text("Закрыть все", 13f, pal.danger, true, Gravity.CENTER).apply {
            setPadding(ctx.dp(12), ctx.dp(10), ctx.dp(12), ctx.dp(10))
            setOnClickListener { closeAll(showPrivate) }
        })
        bottom.addView(View(ctx), weighted())
        bottom.addView(ctx.pillButton("+ Новая вкладка", pal) { newTab(showPrivate) })
        body.addView(ctx.divider(pal))
        body.addView(bottom, matchWrap())
        render()
    }

    private fun render() {
        val all = tabs()
        segRegular.text = "Обычные (${all.count { !it.priv }})"
        segPrivate.text = "Приватные (${all.count { it.priv }})"
        fun style(v: android.widget.TextView, on: Boolean, accent: Int) {
            v.setTextColor(if (on) Color.WHITE else pal.muted)
            v.background = if (on) shape(accent, ctx.dp(10).toFloat()) else null
        }
        style(segRegular, !showPrivate, pal.accent)
        style(segPrivate, showPrivate, 0xFF7E57C2.toInt())
        list.removeAllViews()
        val shown = all.filter { it.priv == showPrivate }
        if (shown.isEmpty()) list.addView(empty("Нет открытых вкладок"))
        shown.forEach { t ->
            val r = row(t.title, t.url, { select(t.id) }, { closeTab(t.id); render() })
            if (t.id == activeId) r.background = ripple(pal.ripple, shape(pal.card, ctx.dp(10).toFloat(), pal.accent, ctx.dp(2)))
            list.addView(r)
        }
    }
}

/** Настройки. */
class SettingsPanel(
    ctx: Context, pal: Pal, private val prefs: Prefs,
    private val onChanged: () -> Unit,
    private val onClearData: () -> Unit,
    private val onSetDefault: () -> Unit,
    private val onProxy: () -> Unit,
    close: () -> Unit,
) : Panel(ctx, pal, "Настройки", close) {

    private val col = LinearLayout(ctx).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(0, 0, 0, ctx.dp(24))
    }

    init {
        body.addView(scroll(col), LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
        render()
    }

    private fun section(t: String) = col.addView(ctx.text(t.uppercase(), 11f, pal.accent, true).apply {
        setPadding(ctx.dp(16), ctx.dp(20), ctx.dp(16), ctx.dp(6))
    })

    private fun choice(title: String, value: String, onClick: () -> Unit) {
        col.addView(LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(ctx.dp(16), ctx.dp(13), ctx.dp(16), ctx.dp(13))
            background = ripple(pal.ripple, null)
            addView(ctx.text(title, 15f, pal.text).apply { layoutParams = weighted() })
            addView(ctx.text(value, 13f, pal.muted))
            setOnClickListener { onClick() }
        })
    }

    private fun toggle(title: String, hint: String?, get: () -> Boolean, set: (Boolean) -> Unit) {
        val sw = android.widget.Switch(ctx).apply { isChecked = get() }
        val row = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(ctx.dp(16), ctx.dp(11), ctx.dp(16), ctx.dp(11))
            background = ripple(pal.ripple, null)
        }
        val txt = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL; layoutParams = weighted().apply { marginEnd = ctx.dp(12) } }
        txt.addView(ctx.text(title, 15f, pal.text))
        if (hint != null) txt.addView(ctx.text(hint, 11f, pal.muted).apply { setPadding(0, ctx.dp(3), 0, 0) })
        row.addView(txt); row.addView(sw)
        row.setOnClickListener { sw.toggle() }
        sw.setOnCheckedChangeListener { _, v -> set(v); onChanged() }
        col.addView(row)
    }

    private fun pick(title: String, items: Array<String>, current: Int, set: (Int) -> Unit) {
        AlertDialog.Builder(ctx).setTitle(title)
            .setSingleChoiceItems(items, current) { d, w -> set(w); d.dismiss(); onChanged(); render() }
            .setNegativeButton("Отмена", null).show()
    }

    private fun render() {
        col.removeAllViews()
        val themes = arrayOf("Светлая", "Тёмная", "AMOLED (чёрная)")
        val engines = arrayOf("DuckDuckGo", "Google", "Яндекс", "Bing")
        val engineIds = arrayOf("duckduckgo", "google", "yandex", "bing")
        val zooms = intArrayOf(80, 90, 100, 115, 130, 150)

        section("Внешний вид")
        choice("Тема", themes[prefs.theme.coerceIn(0, 2)]) { pick("Тема", themes, prefs.theme) { prefs.theme = it } }
        toggle("Адресная строка снизу", "Удобнее одной рукой", { prefs.addressBarBottom }, { prefs.addressBarBottom = it })
        choice("Размер текста на страницах", "${prefs.textZoom}%") {
            pick("Размер текста", zooms.map { "$it%" }.toTypedArray(), zooms.indexOf(prefs.textZoom).coerceAtLeast(2)) { prefs.textZoom = zooms[it] }
        }
        toggle("Тёмные страницы", "Принудительная тёмная тема сайтов (Android 10+)", { prefs.darkPages }, { prefs.darkPages = it })

        section("Поиск")
        choice("Поисковая система", engines[engineIds.indexOf(prefs.searchEngine).coerceAtLeast(0)]) {
            pick("Поисковая система", engines, engineIds.indexOf(prefs.searchEngine).coerceAtLeast(0)) { prefs.searchEngine = engineIds[it] }
        }

        section("Блокировка и приватность")
        toggle("Блокировать рекламу", "Встроенный движок на Rust, без расширений", { prefs.blockAds }, { prefs.blockAds = it })
        toggle("Блокировать трекеры и счётчики", null, { prefs.blockTrackers }, { prefs.blockTrackers = it })
        toggle("Блокировать сторонние cookies", null, { prefs.blockThirdPartyCookies }, { prefs.blockThirdPartyCookies = it })
        toggle("Чистить ссылки от трекинга", "Убирать utm_*, fbclid, gclid при копировании и «Поделиться»", { prefs.stripTracking }, { prefs.stripTracking = it })
        toggle("Очищать данные при выходе", "Кэш, cookies, история", { prefs.clearOnExit }, { prefs.clearOnExit = it })

        section("Память и трафик")
        toggle("Не загружать изображения", "Экономит трафик и оперативную память", { prefs.blockImages }, { prefs.blockImages = it })
        toggle("JavaScript", "Отключение ускоряет текстовые страницы, но ломает многие сайты", { prefs.javaScript }, { prefs.javaScript = it })
        toggle("Версия для ПК", null, { prefs.desktopMode }, { prefs.desktopMode = it })

        section("Сеть")
        choice("Прокси (VLESS, VMess, Trojan, SS)", if (prefs.proxyEnabled) "Включён" else "Выключен") { onProxy() }

        section("Режим чтения")
        choice("Шрифт", if (prefs.readerSerif) "С засечками" else "Без засечек") { prefs.readerSerif = !prefs.readerSerif; render() }

        section("Данные")
        choice("Очистить кэш, cookies и историю", "") {
            AlertDialog.Builder(ctx).setMessage("Удалить историю, cookies, кэш и данные сайтов?")
                .setNegativeButton("Отмена", null)
                .setPositiveButton("Очистить") { _, _ -> onClearData(); Toast.makeText(ctx, "Данные очищены", Toast.LENGTH_SHORT).show() }.show()
        }
        choice("Сделать браузером по умолчанию", "") { onSetDefault() }
        choice("Сбросить настройки", "") {
            AlertDialog.Builder(ctx).setMessage("Вернуть настройки по умолчанию?")
                .setNegativeButton("Отмена", null)
                .setPositiveButton("Сбросить") { _, _ -> prefs.reset(); onChanged(); render() }.show()
        }

        section("О браузере")
        choice("Tyt Browser", "v${BuildConfig.VERSION_NAME}") {}
        choice("Ядро", if (Core.loaded) "Rust ${Core.version()}" else "Kotlin (нативная библиотека не загружена)") {}
        choice("Движок страниц", "Android System WebView") {}
    }
}

/** Профили встроенного прокси: импорт ссылок из буфера, выбор и переключатель. */
class ProxyPanel(
    ctx: Context, pal: Pal, private val prefs: Prefs,
    private val onApply: () -> Unit,
    private val onRules: () -> Unit,
    close: () -> Unit,
) : Panel(ctx, pal, "Прокси", close) {

    private val col = LinearLayout(ctx).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(0, 0, 0, ctx.dp(24))
    }

    init {
        body.addView(scroll(col), LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
        render()
    }

    private fun links(): List<String> = prefs.proxyLinks.split('\n').filter { it.isNotBlank() }

    private fun render() {
        col.removeAllViews()
        val profiles = Core.proxyParse(links().joinToString("\n"))
        val active = prefs.proxyActiveLink

        val sw = android.widget.Switch(ctx).apply { isChecked = prefs.proxyEnabled }
        val row = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(ctx.dp(16), ctx.dp(14), ctx.dp(16), ctx.dp(14))
        }
        val txt = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL; layoutParams = weighted().apply { marginEnd = ctx.dp(12) } }
        txt.addView(ctx.text("Прокси для браузера", 15f, pal.text))
        txt.addView(ctx.text(
            if (!Core.loaded) "Нативное ядро не загружено"
            else if (prefs.proxyEnabled && dev.tytebyte.tytbrowser.ProxyManager.isRunning()) "Работает на 127.0.0.1:${dev.tytebyte.tytbrowser.ProxyManager.PORT}"
            else "Только трафик страниц; загрузки файлов идут напрямую",
            11f, pal.muted
        ).apply { setPadding(0, ctx.dp(3), 0, 0) })
        row.addView(txt); row.addView(sw)
        sw.setOnCheckedChangeListener { _, v ->
            if (v && dev.tytebyte.tytbrowser.ProxyManager.activeLinks(prefs).isEmpty()) {
                sw.isChecked = false
                Toast.makeText(ctx, "Сначала добавьте и выберите профиль", Toast.LENGTH_SHORT).show()
            } else {
                prefs.proxyEnabled = v
                onApply()
                render()
            }
        }
        col.addView(row)

        col.addView(ctx.text("ПРОФИЛИ", 11f, pal.accent, true).apply { setPadding(ctx.dp(16), ctx.dp(16), ctx.dp(16), ctx.dp(6)) })
        if (profiles.isEmpty()) {
            col.addView(ctx.text("Пока пусто. Скопируйте ссылку vless://, vmess://, trojan:// или ss:// и нажмите «Вставить из буфера».", 13f, pal.muted).apply {
                setPadding(ctx.dp(16), ctx.dp(6), ctx.dp(16), ctx.dp(10))
            })
        }
        profiles.forEach { p ->
            val on = p.link == active && !prefs.proxyUseChain
            col.addView(LinearLayout(ctx).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(ctx.dp(16), ctx.dp(12), ctx.dp(16), ctx.dp(12))
                background = ripple(pal.ripple, null)
                val t = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL; layoutParams = weighted() }
                t.addView(ctx.text(p.name, 15f, if (on) pal.accent else pal.text).apply { maxLines = 1; ellipsize = android.text.TextUtils.TruncateAt.END })
                t.addView(ctx.text("${p.protocol.uppercase()} · ${p.address}", 11f, pal.muted).apply { setPadding(0, ctx.dp(3), 0, 0) })
                addView(t)
                if (on) addView(ctx.text("✓", 18f, pal.accent))
                setOnClickListener {
                    prefs.proxyActiveLink = p.link
                    prefs.proxyUseChain = false
                    if (prefs.proxyEnabled) onApply()
                    render()
                }
                setOnLongClickListener {
                    AlertDialog.Builder(ctx).setTitle(p.name)
                        .setItems(arrayOf("Удалить")) { _, _ ->
                            prefs.proxyLinks = links().filter { it != p.link }.joinToString("\n")
                            if (on) { prefs.proxyActiveLink = ""; prefs.proxyEnabled = false; onApply() }
                            render()
                        }.show()
                    true
                }
            })
        }
        col.addView(ctx.text("+ Вставить из буфера", 14f, pal.accent, true).apply {
            setPadding(ctx.dp(16), ctx.dp(16), ctx.dp(16), ctx.dp(8))
            setOnClickListener { importClipboard() }
        })
        col.addView(ctx.text("ЦЕПОЧКА И МАРШРУТЫ", 11f, pal.accent, true).apply { setPadding(ctx.dp(16), ctx.dp(16), ctx.dp(16), ctx.dp(6)) })
        val chainLinks = prefs.proxyChain.lines().filter { it.isNotBlank() }
        val byLink = profiles.associateBy { it.link }
        val chainText = if (chainLinks.isEmpty()) "не задана"
        else chainLinks.joinToString(" → ") { byLink[it]?.name ?: "?" }
        col.addView(LinearLayout(ctx).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(ctx.dp(16), ctx.dp(12), ctx.dp(16), ctx.dp(12))
            background = ripple(pal.ripple, null)
            addView(ctx.text("Цепочка прокси" + if (prefs.proxyUseChain && chainLinks.isNotEmpty()) "  ✓" else "", 15f,
                if (prefs.proxyUseChain && chainLinks.isNotEmpty()) pal.accent else pal.text))
            addView(ctx.text(chainText, 11f, pal.muted).apply { setPadding(0, ctx.dp(3), 0, 0) })
            setOnClickListener { chainMenu(profiles, chainLinks) }
        })
        col.addView(LinearLayout(ctx).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(ctx.dp(16), ctx.dp(12), ctx.dp(16), ctx.dp(12))
            background = ripple(pal.ripple, null)
            val n = RouteRules.decode(prefs.proxyRules).count { it.enabled }
            addView(ctx.text("Правила маршрутизации", 15f, pal.text))
            addView(ctx.text("Включено правил: $n · по умолчанию ${if (prefs.proxyDefaultDirect) "напрямую" else "через прокси"}", 11f, pal.muted)
                .apply { setPadding(0, ctx.dp(3), 0, 0) })
            setOnClickListener { onRules() }
        })
        col.addView(ctx.text("Журнал прокси", 14f, pal.accent, true).apply {
            setPadding(ctx.dp(16), ctx.dp(8), ctx.dp(16), ctx.dp(8))
            setOnClickListener { showLog() }
        })
        col.addView(ctx.text("Оригинальное ядро Xray-core: VLESS, VMess, Trojan, SS; tcp, ws, grpc, httpupgrade, xhttp; TLS и REALITY; flow (Vision). Остальные ссылки пропускаются.", 11f, pal.muted).apply {
            setPadding(ctx.dp(16), ctx.dp(8), ctx.dp(16), 0)
        })
    }

    private fun chainMenu(profiles: List<dev.tytebyte.tytbrowser.core.ProxyProfile>, chainLinks: List<String>) {
        val usable = chainLinks.isNotEmpty()
        val items = mutableListOf("Изменить звенья")
        if (usable) items.add(0, if (prefs.proxyUseChain) "Не использовать цепочку" else "Использовать цепочку")
        if (usable) items.add("Очистить")
        AlertDialog.Builder(ctx).setTitle("Цепочка прокси")
            .setItems(items.toTypedArray()) { _, w ->
                when (items[w]) {
                    "Использовать цепочку" -> { prefs.proxyUseChain = true; if (prefs.proxyEnabled) onApply(); render() }
                    "Не использовать цепочку" -> { prefs.proxyUseChain = false; if (prefs.proxyEnabled) onApply(); render() }
                    "Изменить звенья" -> editChain(profiles, chainLinks)
                    "Очистить" -> {
                        prefs.proxyChain = ""; prefs.proxyUseChain = false
                        if (prefs.proxyEnabled) onApply()
                        render()
                    }
                }
            }.show()
    }

    /** Звенья выбираются по порядку нажатия: первое — вход (ближе к вам), последнее — выход в интернет. */
    private fun editChain(profiles: List<dev.tytebyte.tytbrowser.core.ProxyProfile>, current: List<String>) {
        if (profiles.size < 2) {
            Toast.makeText(ctx, "Для цепочки нужно минимум два профиля", Toast.LENGTH_LONG).show()
            return
        }
        val order = current.filter { l -> profiles.any { it.link == l } }.toMutableList()
        fun labels() = profiles.map { it.name }.toTypedArray()
        val checked = BooleanArray(profiles.size) { profiles[it].link in order }
        lateinit var dialog: AlertDialog
        dialog = AlertDialog.Builder(ctx).setTitle("Звенья по порядку: вход → выход")
            .setMultiChoiceItems(labels(), checked) { _, w, on ->
                val l = profiles[w].link
                if (on) { if (l !in order) order.add(l) } else order.remove(l)
                dialog.setTitle("Звенья: " + if (order.isEmpty()) "ничего не выбрано" else order.joinToString(" → ") { o -> profiles.first { it.link == o }.name })
            }
            .setNegativeButton("Отмена", null)
            .setPositiveButton("Сохранить") { _, _ ->
                if (order.size < 2) {
                    Toast.makeText(ctx, "Выберите минимум два звена", Toast.LENGTH_LONG).show()
                } else {
                    prefs.proxyChain = order.joinToString("\n")
                    prefs.proxyUseChain = true
                    if (prefs.proxyEnabled) onApply()
                    render()
                }
            }.show()
    }

    private fun showLog() {
        val log = Core.proxyLog().ifBlank { "Журнал пуст: прокси ещё не запускался." }
        val tv = ctx.text(log.takeLast(12000), 10f, pal.text).apply {
            typeface = android.graphics.Typeface.MONOSPACE
            setTextIsSelectable(true)
            setPadding(ctx.dp(16), ctx.dp(8), ctx.dp(16), ctx.dp(8))
        }
        AlertDialog.Builder(ctx).setTitle("Журнал прокси").setView(ScrollView(ctx).apply { addView(tv) })
            .setNegativeButton("Закрыть", null)
            .setPositiveButton("Копировать") { _, _ ->
                val cm = ctx.getSystemService(Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
                cm.setPrimaryClip(android.content.ClipData.newPlainText("proxy log", log))
                Toast.makeText(ctx, "Скопировано", Toast.LENGTH_SHORT).show()
            }.show()
    }

    private fun importClipboard() {
        val cm = ctx.getSystemService(Context.CLIPBOARD_SERVICE) as android.content.ClipboardManager
        val text = cm.primaryClip?.takeIf { it.itemCount > 0 }?.getItemAt(0)?.coerceToText(ctx)?.toString().orEmpty()
        val found = Core.proxyParse(text)
        if (found.isEmpty()) {
            Toast.makeText(ctx, "В буфере нет поддерживаемых ссылок", Toast.LENGTH_LONG).show()
            return
        }
        val have = links().toMutableList()
        var added = 0
        found.forEach { if (it.link !in have) { have.add(it.link); added++ } }
        prefs.proxyLinks = have.joinToString("\n")
        if (prefs.proxyActiveLink.isEmpty()) prefs.proxyActiveLink = found.first().link
        Toast.makeText(ctx, "Добавлено профилей: $added", Toast.LENGTH_SHORT).show()
        render()
    }
}

/** Правила маршрутизации: domain/full/keyword/regexp/geosite, IP/CIDR/geoip, порты, сеть; базы geo. */
class RulesPanel(
    ctx: Context, pal: Pal, private val prefs: Prefs,
    private val onApply: () -> Unit,
    close: () -> Unit,
) : Panel(ctx, pal, "Правила маршрутизации", close) {

    private val col = LinearLayout(ctx).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(0, 0, 0, ctx.dp(24))
    }

    init {
        body.addView(scroll(col), LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
        render()
    }

    private fun rules() = RouteRules.decode(prefs.proxyRules)

    private fun save(list: List<RouteRule>) {
        prefs.proxyRules = RouteRules.encode(list)
        if (prefs.proxyEnabled) onApply()
        render()
    }

    private fun render() {
        col.removeAllViews()
        val list = rules()

        col.addView(row("По умолчанию — напрямую", "Трафик, не подошедший ни под одно правило. Выключено: через прокси.",
            android.widget.Switch(ctx).apply {
                isChecked = prefs.proxyDefaultDirect
                setOnCheckedChangeListener { _, v -> prefs.proxyDefaultDirect = v; if (prefs.proxyEnabled) onApply(); render() }
            }))

        col.addView(ctx.text("ПРАВИЛА (сверху вниз, срабатывает первое)", 11f, pal.accent, true).apply {
            setPadding(ctx.dp(16), ctx.dp(16), ctx.dp(16), ctx.dp(6))
        })
        if (list.isEmpty()) {
            col.addView(ctx.text("Правил нет: весь трафик идёт по умолчанию. Добавьте своё или возьмите шаблон.", 13f, pal.muted).apply {
                setPadding(ctx.dp(16), ctx.dp(6), ctx.dp(16), ctx.dp(10))
            })
        }
        list.forEachIndexed { i, r ->
            col.addView(row("${r.name.ifBlank { "Правило ${i + 1}" }} → ${r.action.title}", r.summary,
                android.widget.Switch(ctx).apply {
                    isChecked = r.enabled
                    setOnCheckedChangeListener { _, v -> save(list.toMutableList().also { it[i] = r.copy(enabled = v) }) }
                }).apply {
                setOnClickListener { edit(i, r) }
                setOnLongClickListener {
                    AlertDialog.Builder(ctx).setTitle(r.name)
                        .setItems(arrayOf("Выше", "Ниже", "Удалить")) { _, w ->
                            val m = list.toMutableList()
                            when (w) {
                                0 -> if (i > 0) { m.removeAt(i); m.add(i - 1, r) }
                                1 -> if (i < m.lastIndex) { m.removeAt(i); m.add(i + 1, r) }
                                else -> m.removeAt(i)
                            }
                            save(m)
                        }.show()
                    true
                }
            })
        }
        col.addView(ctx.text("+ Добавить правило", 14f, pal.accent, true).apply {
            setPadding(ctx.dp(16), ctx.dp(14), ctx.dp(16), ctx.dp(6))
            setOnClickListener { edit(-1, RouteRule("")) }
        })
        col.addView(ctx.text("+ Шаблоны (локальные сети, Россия, реклама)", 14f, pal.accent, true).apply {
            setPadding(ctx.dp(16), ctx.dp(10), ctx.dp(16), ctx.dp(6))
            setOnClickListener { presets() }
        })

        col.addView(ctx.text("БАЗЫ GEO", 11f, pal.accent, true).apply { setPadding(ctx.dp(16), ctx.dp(20), ctx.dp(16), ctx.dp(6)) })
        col.addView(ctx.text("geoip.dat и geosite.dat: ${GeoData.status(ctx)}", 13f, pal.text).apply { setPadding(ctx.dp(16), ctx.dp(4), ctx.dp(16), ctx.dp(4)) })
        col.addView(ctx.text("Нужны для правил geoip: и geosite:. Скачиваются напрямую, не через прокси.", 11f, pal.muted).apply {
            setPadding(ctx.dp(16), 0, ctx.dp(16), ctx.dp(6))
        })
        col.addView(ctx.text("Скачать / обновить базы", 14f, pal.accent, true).apply {
            setPadding(ctx.dp(16), ctx.dp(8), ctx.dp(16), ctx.dp(6))
            setOnClickListener {
                text = "Загрузка…"
                GeoData.update(ctx) { err ->
                    Toast.makeText(ctx, if (err == null) "Базы обновлены" else "Не удалось: $err", Toast.LENGTH_LONG).show()
                    if (err == null && prefs.proxyEnabled) onApply()
                    render()
                }
            }
        })
        col.addView(ctx.text("Формат: домен — example.com (с поддоменами), full:, keyword:, regexp:, geosite:ru; IP — 1.2.3.0/24, geoip:ru; порты — 80,443,1000-2000.", 11f, pal.muted).apply {
            setPadding(ctx.dp(16), ctx.dp(16), ctx.dp(16), 0)
        })
    }

    private fun row(title: String, hint: String, trailing: View): LinearLayout = LinearLayout(ctx).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        setPadding(ctx.dp(16), ctx.dp(11), ctx.dp(16), ctx.dp(11))
        background = ripple(pal.ripple, null)
        val t = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL; layoutParams = weighted().apply { marginEnd = ctx.dp(12) } }
        t.addView(ctx.text(title, 15f, pal.text))
        t.addView(ctx.text(hint, 11f, pal.muted).apply { setPadding(0, ctx.dp(3), 0, 0); maxLines = 2; ellipsize = TextUtils.TruncateAt.END })
        addView(t); addView(trailing)
    }

    private fun presets() {
        val ps = RouteRules.presets
        val sel = BooleanArray(ps.size)
        AlertDialog.Builder(ctx).setTitle("Шаблоны правил")
            .setMultiChoiceItems(ps.map { it.name }.toTypedArray(), sel) { _, w, on -> sel[w] = on }
            .setNegativeButton("Отмена", null)
            .setPositiveButton("Добавить") { _, _ ->
                val add = ps.filterIndexed { i, _ -> sel[i] }
                if (add.isNotEmpty()) {
                    save(rules() + add)
                    if (!GeoData.hasAll(ctx)) Toast.makeText(ctx, "Скачайте базы geo, иначе прокси не запустится", Toast.LENGTH_LONG).show()
                }
            }.show()
    }

    private fun edit(index: Int, r: RouteRule) {
        val box = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL; setPadding(ctx.dp(20), ctx.dp(8), ctx.dp(20), 0) }
        fun field(hint: String, value: String, multi: Boolean) = EditText(ctx).apply {
            this.hint = hint; setText(value)
            if (multi) { isSingleLine = false; minLines = 2; maxLines = 6 } else isSingleLine = true
            box.addView(this)
        }
        val name = field("Название", r.name, false)
        val domains = field("Домены (по одному в строке)", r.domains, true)
        val ips = field("IP / CIDR / geoip:xx (по одному в строке)", r.ips, true)
        val ports = field("Порты, например 80,443,1000-2000", r.ports, false)
        val nets = arrayOf("Любая сеть", "TCP", "UDP")
        val net = android.widget.Spinner(ctx).apply {
            adapter = android.widget.ArrayAdapter(ctx, android.R.layout.simple_spinner_dropdown_item, nets)
            setSelection(when (r.network) { "tcp" -> 1; "udp" -> 2; else -> 0 })
        }
        val acts = RouteAction.entries
        val act = android.widget.Spinner(ctx).apply {
            adapter = android.widget.ArrayAdapter(ctx, android.R.layout.simple_spinner_dropdown_item, acts.map { it.title })
            setSelection(acts.indexOf(r.action))
        }
        box.addView(net); box.addView(act)
        AlertDialog.Builder(ctx).setTitle(if (index < 0) "Новое правило" else "Правило")
            .setView(ScrollView(ctx).apply { addView(box) })
            .setNegativeButton("Отмена", null)
            .setPositiveButton("Сохранить") { _, _ ->
                val nr = RouteRule(
                    name.text.toString().trim(), r.enabled || index < 0, acts[act.selectedItemPosition],
                    domains.text.toString().trim(), ips.text.toString().trim(), ports.text.toString().trim(),
                    arrayOf("", "tcp", "udp")[net.selectedItemPosition],
                )
                if (nr.domains.isEmpty() && nr.ips.isEmpty() && nr.ports.isEmpty() && nr.network.isEmpty()) {
                    Toast.makeText(ctx, "Укажите хотя бы одно условие", Toast.LENGTH_LONG).show()
                } else {
                    val m = rules().toMutableList()
                    if (index < 0) m.add(nr) else m[index] = nr
                    save(m)
                }
            }.show()
    }
}
