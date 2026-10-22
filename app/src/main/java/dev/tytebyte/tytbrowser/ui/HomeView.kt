package dev.tytebyte.tytbrowser.ui

import android.app.AlertDialog
import android.content.Context
import android.graphics.Color
import android.text.InputType
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.view.inputmethod.EditorInfo
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import dev.tytebyte.tytbrowser.core.Core
import dev.tytebyte.tytbrowser.data.Bookmark
import dev.tytebyte.tytbrowser.data.Db

/** Стартовая страница: поиск, ярлыки быстрого доступа, переходы в разделы и счётчик блокировок. */
class HomeView(
    private val ctx: Context,
    private val pal: Pal,
    private val db: Db,
    private val engineName: String,
    private val blockedTotal: Long,
    private val isPrivate: Boolean,
    private val onOpen: (String) -> Unit,
    private val onPickEngine: () -> Unit,
    private val onSection: (String) -> Unit,
    private val onChanged: () -> Unit,
) {
    val view: View

    init {
        val col = LinearLayout(ctx).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER_HORIZONTAL
            setPadding(ctx.dp(16), ctx.dp(28), ctx.dp(16), ctx.dp(28))
        }

        // Заголовок
        col.addView(ctx.text("Tyt Browser", 26f, pal.text, true, Gravity.CENTER).apply {
            layoutParams = matchWrap()
        })
        col.addView(ctx.text(
            if (isPrivate) "Приватная вкладка: история не сохраняется" else "Быстрый и лёгкий браузер",
            12f, pal.muted, false, Gravity.CENTER
        ).apply { layoutParams = matchWrap().apply { topMargin = ctx.dp(4); bottomMargin = ctx.dp(22) } })

        // Строка поиска
        val searchRow = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            background = shape(pal.field, ctx.dp(26).toFloat(), pal.border, ctx.dp(1))
            setPadding(ctx.dp(6), 0, ctx.dp(14), 0)
            layoutParams = matchWrap().apply { bottomMargin = ctx.dp(18) }
        }
        val engineChip = ctx.text("$engineName ▾", 12f, pal.accent, true, Gravity.CENTER).apply {
            setPadding(ctx.dp(12), ctx.dp(8), ctx.dp(12), ctx.dp(8))
            background = ripple(pal.ripple, shape(Color.TRANSPARENT, ctx.dp(18).toFloat()), shape(Color.WHITE, ctx.dp(18).toFloat()))
            setOnClickListener { onPickEngine() }
        }
        val input = EditText(ctx).apply {
            hint = "Поиск или веб-адрес"
            setHintTextColor(pal.muted)
            setTextColor(pal.text)
            textSize = 15f
            background = null
            maxLines = 1
            isSingleLine = true
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_URI
            imeOptions = EditorInfo.IME_ACTION_GO
            setPadding(ctx.dp(8), ctx.dp(14), ctx.dp(8), ctx.dp(14))
            layoutParams = weighted()
            setOnEditorActionListener { v, _, _ ->
                val q = v.text.toString().trim()
                if (q.isNotEmpty()) onOpen(q)
                true
            }
        }
        searchRow.addView(engineChip)
        searchRow.addView(input)
        col.addView(searchRow)

        // Статистика
        col.addView(ctx.text(
            "Заблокировано рекламы и трекеров: $blockedTotal  ·  ядро: ${if (Core.loaded) "Rust ${Core.version()}" else "Kotlin"}",
            11f, pal.muted, false, Gravity.CENTER
        ).apply { layoutParams = matchWrap().apply { bottomMargin = ctx.dp(18) } })

        // Ярлыки
        val bookmarks = db.bookmarks()
        val head = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            layoutParams = matchWrap().apply { bottomMargin = ctx.dp(10) }
        }
        head.addView(ctx.text("БЫСТРЫЙ ДОСТУП", 11f, pal.muted, true).apply { layoutParams = weighted() })
        head.addView(ctx.text("+ Добавить", 12f, pal.accent, true).apply {
            setPadding(ctx.dp(8), ctx.dp(4), 0, ctx.dp(4))
            setOnClickListener { addDialog() }
        })
        col.addView(head)

        val cols = 4
        val tiles = bookmarks.map { tile(it) } + listOf(addTile())
        tiles.chunked(cols).forEach { rowTiles ->
            val row = LinearLayout(ctx).apply {
                orientation = LinearLayout.HORIZONTAL
                layoutParams = matchWrap().apply { bottomMargin = ctx.dp(12) }
            }
            rowTiles.forEach { row.addView(it) }
            repeat(cols - rowTiles.size) { row.addView(View(ctx).apply { layoutParams = weighted() }) }
            col.addView(row)
        }

        // Разделы
        val sections = LinearLayout(ctx).apply {
            orientation = LinearLayout.HORIZONTAL
            layoutParams = matchWrap().apply { topMargin = ctx.dp(10) }
        }
        listOf("★" to "Закладки", "⏱" to "История", "⬇" to "Загрузки", "⚙" to "Настройки").forEach { (g, l) ->
            sections.addView(LinearLayout(ctx).apply {
                orientation = LinearLayout.VERTICAL
                gravity = Gravity.CENTER
                setPadding(0, ctx.dp(12), 0, ctx.dp(10))
                layoutParams = weighted().apply { marginStart = ctx.dp(3); marginEnd = ctx.dp(3) }
                background = ripple(pal.ripple, shape(pal.card, ctx.dp(14).toFloat(), pal.border, ctx.dp(1)), shape(Color.WHITE, ctx.dp(14).toFloat()))
                addView(ctx.text(g, 20f, pal.accent, false, Gravity.CENTER))
                addView(ctx.text(l, 11f, pal.text, false, Gravity.CENTER).apply { setPadding(0, ctx.dp(6), 0, 0) })
                setOnClickListener { onSection(l) }
            })
        }
        col.addView(sections)

        view = ScrollView(ctx).apply {
            setBackgroundColor(pal.bg)
            isFillViewport = true
            addView(col, ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT))
        }
    }

    private fun tile(b: Bookmark): View = LinearLayout(ctx).apply {
        orientation = LinearLayout.VERTICAL
        gravity = Gravity.CENTER_HORIZONTAL
        layoutParams = weighted()
        val letter = (Db.hostOf(b.url).firstOrNull() ?: '?').uppercaseChar().toString()
        addView(ctx.text(letter, 20f, Color.WHITE, true, Gravity.CENTER).apply {
            layoutParams = LinearLayout.LayoutParams(ctx.dp(54), ctx.dp(54))
            background = ripple(0x33FFFFFF, shape(b.color, ctx.dp(16).toFloat()), shape(Color.WHITE, ctx.dp(16).toFloat()))
        })
        addView(ctx.text(b.title, 11f, pal.text, false, Gravity.CENTER).apply {
            maxLines = 1
            ellipsize = android.text.TextUtils.TruncateAt.END
            layoutParams = LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT).apply { topMargin = ctx.dp(6) }
            setPadding(ctx.dp(2), 0, ctx.dp(2), 0)
        })
        setOnClickListener { onOpen(b.url) }
        setOnLongClickListener {
            AlertDialog.Builder(ctx)
                .setTitle(b.title)
                .setItems(arrayOf("Удалить ярлык")) { _, _ -> db.removeBookmark(b.id); onChanged() }
                .show()
            true
        }
    }

    private fun addTile(): View = LinearLayout(ctx).apply {
        orientation = LinearLayout.VERTICAL
        gravity = Gravity.CENTER_HORIZONTAL
        layoutParams = weighted()
        addView(ctx.text("+", 24f, pal.muted, false, Gravity.CENTER).apply {
            layoutParams = LinearLayout.LayoutParams(ctx.dp(54), ctx.dp(54))
            background = shape(Color.TRANSPARENT, ctx.dp(16).toFloat(), pal.border, ctx.dp(2))
        })
        addView(ctx.text("Новый", 11f, pal.muted, false, Gravity.CENTER).apply { setPadding(0, ctx.dp(6), 0, 0) })
        setOnClickListener { addDialog() }
    }

    private fun addDialog() {
        val box = LinearLayout(ctx).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(ctx.dp(20), ctx.dp(8), ctx.dp(20), 0)
        }
        val title = EditText(ctx).apply { hint = "Название"; isSingleLine = true }
        val url = EditText(ctx).apply {
            hint = "Адрес, например example.com"
            isSingleLine = true
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_URI
        }
        box.addView(title); box.addView(url)
        AlertDialog.Builder(ctx)
            .setTitle("Новый ярлык")
            .setView(box)
            .setNegativeButton("Отмена", null)
            .setPositiveButton("Сохранить") { _, _ ->
                val u = url.text.toString().trim()
                if (u.isNotEmpty()) {
                    val r = Core.resolve(u, "duckduckgo")
                    val finalUrl = if (r.kind == dev.tytebyte.tytbrowser.core.InputKind.URL) r.url else "https://$u"
                    db.addBookmark(title.text.toString().trim().ifEmpty { Db.hostOf(finalUrl) }, finalUrl)
                    onChanged()
                }
            }
            .show()
    }
}
