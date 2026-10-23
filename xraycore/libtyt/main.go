// Пакет libtyt собирается в libtytxray.so (-buildmode=c-shared) и запускает оригинальное ядро Xray-core
// внутри приложения. Java-функции живут в jni_shim.c и вызывают экспортируемые ниже Tyt*-функции.
package main

import "C"

import (
	"net"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/xtls/xray-core/common/log"
	"github.com/xtls/xray-core/core"
	"github.com/xtls/xray-core/infra/conf/serial"

	// Все протоколы, транспорты и функции ядра (VLESS, VMess, Trojan, SS, REALITY, XTLS-Vision, …).
	_ "github.com/xtls/xray-core/main/distro/all"
)

const logCap = 256 * 1024

var (
	mu      sync.Mutex
	server  *core.Instance
	port    int
	logLock sync.Mutex
	logBuf  strings.Builder
)

// ringHandler складывает сообщения ядра в буфер ограниченного размера.
type ringHandler struct{}

func (ringHandler) Handle(msg log.Message) {
	text := msg.String()
	// Отладочный шум и проба готовности порта не нужны: оставляем Info и выше.
	if strings.Contains(text, "[Debug]") || strings.Contains(text, "failed to read http request > EOF") {
		return
	}
	appendLog(text)
}

func appendLog(s string) {
	logLock.Lock()
	defer logLock.Unlock()
	logBuf.WriteString(time.Now().Format("15:04:05 "))
	logBuf.WriteString(s)
	logBuf.WriteByte('\n')
	if logBuf.Len() > logCap {
		t := logBuf.String()
		t = t[len(t)-logCap/2:]
		logBuf.Reset()
		logBuf.WriteString(t)
	}
}

func stopLocked() {
	if server != nil {
		_ = server.Close()
		server = nil
		port = 0
	}
}

func start(cfgJSON string, listenPort int, assetDir string) string {
	mu.Lock()
	defer mu.Unlock()
	stopLocked()
	if assetDir != "" {
		// Базы geoip.dat / geosite.dat ядро ищет в этой папке.
		_ = os.Setenv("XRAY_LOCATION_ASSET", assetDir)
	}
	logLock.Lock()
	logBuf.Reset()
	logLock.Unlock()

	cfg, err := serial.LoadJSONConfig(strings.NewReader(cfgJSON))
	if err != nil {
		return "конфигурация: " + err.Error()
	}
	s, err := core.New(cfg)
	if err != nil {
		return "создание ядра: " + err.Error()
	}
	if err := s.Start(); err != nil {
		_ = s.Close()
		return "запуск ядра: " + err.Error()
	}
	log.RegisterHandler(ringHandler{})
	// Убеждаемся, что локальный порт действительно принимает соединения.
	addr := net.JoinHostPort("127.0.0.1", itoa(listenPort))
	deadline := time.Now().Add(3 * time.Second)
	for {
		c, err := net.DialTimeout("tcp", addr, 200*time.Millisecond)
		if err == nil {
			_ = c.Close()
			break
		}
		if time.Now().After(deadline) {
			_ = s.Close()
			return "ядро не открыло локальный порт " + addr
		}
		time.Sleep(50 * time.Millisecond)
	}
	server = s
	port = listenPort
	appendLog("[tytcore] Xray-core " + core.Version() + " слушает " + addr)
	return ""
}

func itoa(n int) string {
	if n == 0 {
		return "0"
	}
	var b [12]byte
	i := len(b)
	for n > 0 {
		i--
		b[i] = byte('0' + n%10)
		n /= 10
	}
	return string(b[i:])
}

//export TytStart
func TytStart(cfg *C.char, listenPort C.int, assetDir *C.char) *C.char {
	return C.CString(start(C.GoString(cfg), int(listenPort), C.GoString(assetDir)))
}

//export TytStop
func TytStop() {
	mu.Lock()
	defer mu.Unlock()
	stopLocked()
}

//export TytPort
func TytPort() C.int {
	mu.Lock()
	defer mu.Unlock()
	return C.int(port)
}

//export TytLog
func TytLog() *C.char {
	logLock.Lock()
	defer logLock.Unlock()
	return C.CString(logBuf.String())
}

func main() {}
