package dev.tytebyte.tytbrowser.ui

import org.json.JSONObject

/** Собирает HTML режима чтения из JSON, который вернуло Rust-ядро. JavaScript в читалке отключён. */
object ReaderPage {
    private data class Colors(val bg: String, val fg: String, val muted: String, val line: String)

    private val themes = listOf(
        Colors("#FAF9F5", "#2D2D2D", "#77736B", "#DAD6CC"), // светлая бумага
        Colors("#F4ECD8", "#3D332A", "#85796A", "#DDD0B5"), // сепия
        Colors("#1E1E1E", "#D8D8D8", "#8A8A8A", "#383838"), // тёмная
        Colors("#000000", "#B0B0B0", "#707070", "#222222"), // чёрная
    )

    fun defaultTheme(appTheme: Int) = when (appTheme) { 2 -> 3; 1 -> 2; else -> 1 }

    private fun esc(s: String): String {
        val sb = StringBuilder(s.length + 16)
        for (c in s) when (c) {
            '&' -> sb.append("&amp;")
            '<' -> sb.append("&lt;")
            '>' -> sb.append("&gt;")
            '"' -> sb.append("&quot;")
            '\n' -> sb.append("<br>")
            else -> sb.append(c)
        }
        return sb.toString()
    }

    fun build(article: JSONObject, fontPx: Int, theme: Int, serif: Boolean): String {
        val c = themes[theme.coerceIn(0, themes.size - 1)]
        val family = if (serif) "Georgia, 'Noto Serif', serif" else "system-ui, 'Roboto', sans-serif"
        val words = article.optInt("words")
        val minutes = maxOf(1, (words + 179) / 180)
        val sb = StringBuilder(8192)
        sb.append("<!doctype html><html><head><meta charset=\"utf-8\">")
        sb.append("<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">")
        sb.append("<style>")
        sb.append("html{-webkit-text-size-adjust:100%}")
        sb.append("body{margin:0;padding:18px 20px 48px;background:${c.bg};color:${c.fg};font:${fontPx}px/1.7 $family;word-wrap:break-word;overflow-wrap:anywhere}")
        sb.append("main{max-width:40em;margin:0 auto}")
        sb.append("h1{font-size:1.45em;line-height:1.25;margin:.2em 0 .4em}")
        sb.append("h2,h3,h4,h5,h6{line-height:1.3;margin:1.4em 0 .4em}h2{font-size:1.25em}h3{font-size:1.12em}h4,h5,h6{font-size:1em}")
        sb.append("p{margin:.8em 0}ul{padding-left:1.3em}li{margin:.3em 0}")
        sb.append(".meta{font:12px/1.4 system-ui,sans-serif;color:${c.muted};padding-bottom:.9em;border-bottom:1px solid ${c.line};margin-bottom:1em}")
        sb.append("blockquote{margin:1em 0;padding:.1em 0 .1em 1em;border-left:3px solid ${c.line};font-style:italic;opacity:.92}")
        sb.append("pre{overflow-x:auto;padding:.7em .8em;background:${c.line};border-radius:8px;font:13px/1.5 monospace;white-space:pre}")
        sb.append(".cap{font:12px/1.4 system-ui,sans-serif;color:${c.muted};margin:-.3em 0 1em}")
        sb.append("</style></head><body><main>")

        val title = article.optString("title")
        if (title.isNotBlank()) sb.append("<h1>").append(esc(title)).append("</h1>")
        sb.append("<div class=\"meta\">")
        val byline = article.optString("byline")
        if (byline.isNotBlank()) sb.append(esc(byline)).append(" · ")
        sb.append(esc(article.optString("site"))).append(" · ").append(minutes).append(" мин чтения</div>")

        val blocks = article.optJSONArray("blocks")
        var inList = false
        var skippedTitle = false
        if (blocks != null) {
            for (i in 0 until blocks.length()) {
                val b = blocks.getJSONObject(i)
                val t = b.optString("t")
                val x = b.optString("x")
                if (inList && t != "li") { sb.append("</ul>"); inList = false }
                when (t) {
                    "h" -> {
                        // Заголовок статьи уже выведен сверху — не дублируем.
                        if (!skippedTitle && x.trim() == title.trim()) { skippedTitle = true } else {
                            val l = b.optInt("l", 2).coerceIn(2, 6)
                            sb.append("<h$l>").append(esc(x)).append("</h$l>")
                        }
                    }
                    "li" -> { if (!inList) { sb.append("<ul>"); inList = true }; sb.append("<li>").append(esc(x)).append("</li>") }
                    "q" -> sb.append("<blockquote>").append(esc(x)).append("</blockquote>")
                    "pre" -> sb.append("<pre>").append(esc(x).replace("<br>", "\n")).append("</pre>")
                    "cap" -> sb.append("<div class=\"cap\">").append(esc(x)).append("</div>")
                    else -> sb.append("<p>").append(esc(x)).append("</p>")
                }
            }
        }
        if (inList) sb.append("</ul>")
        sb.append("</main></body></html>")
        return sb.toString()
    }

    /** Страница ошибки загрузки (без внешних ресурсов). */
    fun errorPage(pal: Pal, title: String, details: String, url: String): String {
        val bg = if (pal.dark) "#171717" else "#FFFFFF"
        val fg = if (pal.dark) "#F2F2F2" else "#1B1B1B"
        val muted = if (pal.dark) "#9A9A9A" else "#6B6B6B"
        return "<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">" +
            "<style>body{margin:0;padding:48px 24px;background:$bg;color:$fg;font:16px/1.5 system-ui,sans-serif;text-align:center}" +
            "h1{font-size:20px;margin:16px 0 8px}p{color:$muted;font-size:14px;word-break:break-all}" +
            ".i{font-size:56px}a{display:inline-block;margin-top:24px;padding:10px 22px;border-radius:12px;background:#3B82F6;color:#fff;text-decoration:none;font-weight:600}</style></head>" +
            "<body><div class=\"i\">⚠</div><h1>${esc(title)}</h1><p>${esc(details)}</p><p>${esc(url)}</p><a href=\"${esc(url)}\">Повторить</a></body></html>"
    }
}
