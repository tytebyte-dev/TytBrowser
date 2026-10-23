package dev.tytebyte.tytbrowser.data

import org.json.JSONArray
import org.json.JSONObject

/** Куда отправить трафик, подошедший под правило. */
enum class RouteAction(val id: String, val title: String) {
    PROXY("proxy", "Через прокси"),
    DIRECT("direct", "Напрямую"),
    BLOCK("block", "Блокировать");

    companion object {
        fun of(id: String?) = entries.firstOrNull { it.id == id } ?: PROXY
    }
}

/**
 * Правило маршрутизации. [domains] и [ips] — по одному значению в строке.
 * Домены: `example.com` (с поддоменами), `full:`, `keyword:`, `regexp:`, `geosite:ru`.
 * IP: адрес, CIDR или `geoip:ru`. [ports]: `80,443,1000-2000`. [network]: пусто, `tcp` или `udp`.
 */
data class RouteRule(
    val name: String,
    val enabled: Boolean = true,
    val action: RouteAction = RouteAction.PROXY,
    val domains: String = "",
    val ips: String = "",
    val ports: String = "",
    val network: String = "",
) {
    val summary: String
        get() = listOf(domains, ips, ports, network)
            .flatMap { it.lines() }.map(String::trim).filter(String::isNotEmpty).take(3).joinToString(", ").ifEmpty { "без условий" }
}

object RouteRules {
    private const val ITEM_SEP = '\u001e'

    /** Готовые наборы правил. Для geosite/geoip нужны базы geo (скачиваются в панели правил). */
    val presets: List<RouteRule> = listOf(
        RouteRule("Локальные сети — напрямую", action = RouteAction.DIRECT, ips = "geoip:private"),
        RouteRule("Россия — напрямую", action = RouteAction.DIRECT, domains = "geosite:category-ru\nregexp:\\.ru$\nregexp:\\.su$", ips = "geoip:ru"),
        RouteRule("Реклама — блокировать", action = RouteAction.BLOCK, domains = "geosite:category-ads-all"),
    )

    fun encode(rules: List<RouteRule>): String = JSONArray().also { arr ->
        rules.forEach {
            arr.put(JSONObject().put("name", it.name).put("on", it.enabled).put("action", it.action.id)
                .put("domains", it.domains).put("ips", it.ips).put("ports", it.ports).put("network", it.network))
        }
    }.toString()

    fun decode(json: String): List<RouteRule> = try {
        val a = JSONArray(json.ifBlank { "[]" })
        (0 until a.length()).map {
            val o = a.getJSONObject(it)
            RouteRule(o.optString("name"), o.optBoolean("on", true), RouteAction.of(o.optString("action")),
                o.optString("domains"), o.optString("ips"), o.optString("ports"), o.optString("network"))
        }
    } catch (_: Exception) { emptyList() }

    /**
     * Текст для нативного ядра: одно правило на строку, поля через TAB (действие, домены, ip, порты, сеть),
     * элементы внутри поля — через U+001E. Выключенные правила пропускаются.
     */
    fun coreText(rules: List<RouteRule>): String = rules.filter { it.enabled }.joinToString("\n") { r ->
        fun field(v: String) = v.lines().map(String::trim).filter(String::isNotEmpty).joinToString(ITEM_SEP.toString())
        listOf(r.action.id, field(r.domains), field(r.ips), r.ports.trim().replace('\n', ',').replace(" ", ""), r.network.trim())
            .joinToString("\t")
    }
}
