# Практика 4. Прикладной уровень

## Программирование сокетов: Прокси-сервер
Разработайте прокси-сервер для проксирования веб-страниц. 
Приложите скрины, демонстрирующие работу прокси-сервера. 

### Запуск прокси-сервера
Запустите свой прокси-сервер из командной строки, а затем запросите веб-страницу с помощью
вашего браузера. Направьте запросы на прокси-сервер, используя свой IP-адрес и номер порта.
Например, http://localhost:8888/www.google.com

_(*) Вы должны заменить стоящий здесь 8888 на номер порта в серверном коде, 
то есть тот, на котором прокси-сервер слушает запросы._

Вы можете также настроить непосредственно веб-браузер на использование вашего прокси сервера. 
В настройках браузера вам нужно будет указать адрес прокси-сервера и номер порта,
который вы использовали при запуске прокси-сервера (опционально).

### А. Прокси-сервер без кеширования (4 балла)
1. Разработайте свой прокси-сервер для проксирования http GET запросов от клиента веб-серверу 
   с журналированием проксируемых HTTP-запросов. В файле журнала сохраняется
   краткая информация о проксируемых запросах (URL и код ответа). Кеширование в этом
   задании не требуется. **(2 балла)**
2. Добавьте в ваш прокси-сервер обработку ошибок. Отсутствие обработчика ошибок может
   вызвать проблемы. Особенно, когда клиент запрашивает объект, который не доступен, так
   как ответ 404 Not Found, как правило, не имеет тела, а прокси-сервер предполагает, что
   тело есть и пытается прочитать его. **(1 балл)**
3. Простой прокси-сервер поддерживает только метод GET протокола HTTP. Добавьте
   поддержку метода POST. В запросах теперь будет использоваться также тело запроса
   (body). Для вызова POST запросов вы можете использовать Postman. **(1 балл)**

Приложите скрины или логи работы сервера.

#### Демонстрация работы
Проксирование существующего адреса
```
❯ curl http://localhost:8000/neverssl.com
<html>
        <head>
                <title>NeverSSL - Connecting ... </title>
                <style>
                body {
                        font-family: Montserrat, helvetica, arial, sans-serif;
                        font-size: 16x;
                        color: #444444;
                        margin: 0;
                }
                h2 {
                        font-weight: 700;
                        font-size: 1.6em;
                        margin-top: 30px;
                }
                p {
                        line-height: 1.6em;
                }
                .container {
                        max-width: 650px;
                        margin: 20px auto 20px auto;
                        padding-left: 15px;
                        padding-right: 15px
                }
                .header {
                        background-color: #42C0FD;
                        color: #FFFFFF;
                        padding: 10px 0 10px 0;
                        font-size: 2.2em;
                }
                .notice {
                        background-color: red;
                        color: white;
                        padding: 10px 0 10px 0;
                        font-size: 1.25em;
                        animation: flash 4s infinite;
                }
                @keyframes flash {
                0% {
                        background-color: red;
                }
                50% {
                        background-color: #AA0000;
                }
                0% {
                        background-color: red;
                }
                }
                <!-- CSS from Mark Webster https://gist.github.com/markcwebster/9bdf30655cdd5279bad13993ac87c85d -->
                </style>

                <script>
                        var adjectives = [ 'cool' , 'calm' , 'relaxed', 'soothing', 'serene', 'slow',
                                                        'beautiful', 'wonderful', 'wonderous', 'fun', 'good',
                                                        'glowing', 'inner', 'grand', 'majestic', 'astounding',
                                                        'fine', 'splendid', 'transcendent', 'sublime', 'whole',
                                                        'unique', 'old', 'young', 'fresh', 'clear', 'shiny',
                                                        'shining', 'lush', 'quiet', 'bright', 'silver' ];

                        var nouns =       [ 'day', 'dawn', 'peace', 'smile', 'love', 'zen', 'laugh',
                                                        'yawn', 'poem', 'song', 'joke', 'verse', 'kiss', 'sunrise',
                                                        'sunset', 'eclipse', 'moon', 'rainbow', 'rain', 'plan',
                                                        'play', 'chart', 'birds', 'stars', 'pathway', 'secret',
                                                        'treasure', 'melody', 'magic', 'spell', 'light', 'morning'];

                        var prefix =
                                        // Choose 3 zen adjectives
                                        adjectives.sort(function(){return 0.5-Math.random()}).slice(-3).join('')
                                        +
                                        // Coupled with a zen noun
                                        nouns.sort(function(){return 0.5-Math.random()}).slice(-1).join('');
                        window.location.href = 'http://' + prefix + '.neverssl.com/online';
                </script>
        </head>
        <body>
        <noscript>
                <div class="notice">
                        <div class="container">
                                ⚠️ JavaScript appears to be disabled. NeverSSL's cache-busting works better if you enable JavaScript for <code>neverssl.com</code>.
                        </div>
                </div>
        </noscript>
        <div class="header">
                <div class="container">
                <h1>NeverSSL</h1>
                </div>
        </div>
        <div class="content">
        <div class="container">

        <h1 id="status"></h1>
        <script>document.querySelector("#status").textContent = "Connecting ...";</script>
        <noscript>

                <h2>What?</h2>
                <p>This website is for when you try to open Facebook, Google, Amazon, etc
                on a wifi network, and nothing happens. Type "http://neverssl.com"
                into your browser's url bar, and you'll be able to log on.</p>

                <h2>How?</h2>
                <p>neverssl.com will never use SSL (also known as TLS). No
                encryption, no strong authentication, no <a
                href="https://en.wikipedia.org/wiki/HTTP_Strict_Transport_Security">HSTS</a>,
                no HTTP/2.0, just plain old unencrypted HTTP and forever stuck in the dark
                ages of internet security.</p>

                <h2>Why?</h2>
                <p>Normally, that's a bad idea. You should always use SSL and secure
                encryption when possible. In fact, it's such a bad idea that most websites
                are now using https by default.</p>

                <p>And that's great, but it also means that if you're relying on
                poorly-behaved wifi networks, it can be hard to get online.  Secure
                browsers and websites using https make it impossible for those wifi
                networks to send you to a login or payment page. Basically, those networks
                can't tap into your connection just like attackers can't. Modern browsers
                are so good that they can remember when a website supports encryption and
                even if you type in the website name, they'll use https.</p>

                <p>And if the network never redirects you to this page, well as you can
                see, you're not missing much.</p>

        <a href="https://twitter.com/neverssl">Follow @neverssl</a>

        </noscript>

        </div>
        </div>

        </body>
</html>
```

Проксирование несуществующего адреса
```
❯ curl http://localhost:8000/neverssl.c
Failed: error sending request for url (http://neverssl.c/)
```

POST метод:
```
❯ curl -v http://localhost:8000/httpbin.org/post \
  -H "Content-Type: application/json" \
  -d '{"hello":"world"}'
* Host localhost:8000 was resolved.
* IPv6: ::1
* IPv4: 127.0.0.1
*   Trying [::1]:8000...
* connect to ::1 port 8000 from ::1 port 50756 failed: Connection refused
*   Trying 127.0.0.1:8000...
* Connected to localhost (127.0.0.1) port 8000
> POST /httpbin.org/post HTTP/1.1
> Host: localhost:8000
> User-Agent: curl/8.7.1
> Accept: */*
> Content-Type: application/json
> Content-Length: 17
>
* upload completely sent off: 17 bytes
< HTTP/1.1 200 OK
< Content-Length: 427
< Connection: close
<
{
  "args": {},
  "data": "{\"hello\":\"world\"}",
  "files": {},
  "form": {},
  "headers": {
    "Accept": "*/*",
    "Content-Length": "17",
    "Content-Type": "application/json",
    "Host": "localhost",
    "User-Agent": "curl/8.7.1",
    "X-Amzn-Trace-Id": "Root=1-69c1746e-6af7eb801e7176836215c7ce"
  },
  "json": {
    "hello": "world"
  },
  "origin": "193.58.120.106",
  "url": "http://localhost/post"
}
* Closing connection
```

Лог сервера
```
[1774285920] >> [user->proxy] GET /neverssl.com
[1774285920] >>   Host: localhost:8000
[1774285920] >>   User-Agent: curl/8.7.1
[1774285920] >>   Accept: */*
[1774285920] >> [proxy->server] GET http://neverssl.com
[1774285924] << [server->proxy] 200 OK
[1774285924] <<   date: Mon, 23 Mar 2026 17:12:04 GMT
[1774285924] <<   server: Apache/2.4.66 ()
[1774285924] <<   upgrade: h2,h2c
[1774285924] <<   connection: Upgrade
[1774285924] <<   last-modified: Wed, 29 Jun 2022 00:23:33 GMT
[1774285924] <<   etag: "f79-5e28b29d38e93"
[1774285924] <<   accept-ranges: bytes
[1774285924] <<   content-length: 3961
[1774285924] <<   vary: Accept-Encoding
[1774285924] <<   content-type: text/html; charset=UTF-8
[1774285924] << [proxy->user] 200 OK body: 3961 bytes
[1774285928] >> [user->proxy] GET /neverssl.c
[1774285928] >>   Host: localhost:8000
[1774285928] >>   User-Agent: curl/8.7.1
[1774285928] >>   Accept: */*
[1774285928] >> [proxy->server] GET http://neverssl.c
[1774285928] << [proxy->user] 502 Bad Gateway: error sending request for url (http://neverssl.c/)
[1774285934] >> [user->proxy] POST /httpbin.org/post
[1774285934] >>   Host: localhost:8000
[1774285934] >>   User-Agent: curl/8.7.1
[1774285934] >>   Accept: */*
[1774285934] >>   Content-Type: application/json
[1774285934] >>   Content-Length: 17
[1774285934] >>   body: 17 bytes
[1774285934] >> [proxy->server] POST http://httpbin.org/post
[1774285935] << [server->proxy] 200 OK
[1774285935] <<   date: Mon, 23 Mar 2026 17:12:14 GMT
[1774285935] <<   content-type: application/json
[1774285935] <<   content-length: 427
[1774285935] <<   connection: keep-alive
[1774285935] <<   server: gunicorn/19.9.0
[1774285935] <<   access-control-allow-origin: *
[1774285935] <<   access-control-allow-credentials: true
[1774285935] << [proxy->user] 200 OK body: 427 bytes
```

### Б. Прокси-сервер с кешированием (4 балла)
Когда прокси-сервер получает запрос, он проверяет, есть ли запрашиваемый объект в кэше, и,
если да, то возвращает объект из кэша без соединения с веб-сервером. Если объекта в кэше нет,
прокси-сервер извлекает его с веб-сервера обычным GET запросом, возвращает клиенту и
кэширует копию для будущих запросов.

Для проверки того, прокис объект в кеше или нет, необходимо использовать условный GET
запрос. В таком случае вам необходимо указывать в заголовке запроса значение для If-Modified-Since и If-None-Match. 
Подробности можно найти [тут](https://ruturajv.wordpress.com/2005/12/27/conditional-get-request).

Будем считать, что кеш-память прокси-сервера хранится на его жестком диске. Ваш прокси-сервер
должен уметь записывать ответы в кеш и извлекать данные из кеша (т.е. с диска) в случае
попадания в кэш при запросе. Для этого необходимо реализовать некоторую внутреннюю
структуру данных, чтобы отслеживать, какие объекты закешированы.

Приложите скрины или логи, из которых понятно, что ответ на повторный запрос был взят из кэша.

#### Демонстрация работы
3 раза с помощью curl обращался к neverssl.com. В логе написано, что значение взято из кэша при втором обращении, также в этом можно убелиться по времени работы в сравнении с первым запросом (время записано в квадратных скобках). Перед 3 запросом я подождял пока копия в кэше протухнет. Видно что в 3 раз прокси сходил на сервер, получил 304 и уже только тогда ответил значением из кэша.
```
[1774289580] >> [user->proxy] GET /neverssl.com
[1774289580] >>   Host: localhost:8000
[1774289580] >>   User-Agent: curl/8.7.1
[1774289580] >>   Accept: */*
[1774289580] >> [proxy->server] GET http://neverssl.com
[1774289585] << [server->proxy] 200 OK
[1774289585] <<   date: Mon, 23 Mar 2026 18:13:04 GMT
[1774289585] <<   server: Apache/2.4.66 ()
[1774289585] <<   upgrade: h2,h2c
[1774289585] <<   connection: Upgrade
[1774289585] <<   last-modified: Wed, 29 Jun 2022 00:23:33 GMT
[1774289585] <<   etag: "f79-5e28b29d38e93"
[1774289585] <<   accept-ranges: bytes
[1774289585] <<   content-length: 3961
[1774289585] <<   vary: Accept-Encoding
[1774289585] <<   content-type: text/html; charset=UTF-8
[1774289585] << [proxy->user] 200 OK body: 3961 bytes
[1774289591] >> [user->proxy] GET /neverssl.com
[1774289591] >>   Host: localhost:8000
[1774289591] >>   User-Agent: curl/8.7.1
[1774289591] >>   Accept: */*
[1774289591] << [proxy->user] 200 OK (cached)
[1774289591] <<   body: 3961 bytes
[1774289737] >> [proxy->server] GET http://neverssl.com
[1774289739] << [server->proxy] 304 Not Modified
[1774289739] <<   date: Mon, 23 Mar 2026 18:15:39 GMT
[1774289739] <<   server: Apache/2.4.66 ()
[1774289739] <<   upgrade: h2,h2c
[1774289739] <<   connection: Upgrade
[1774289739] <<   last-modified: Wed, 29 Jun 2022 00:23:33 GMT
[1774289739] <<   etag: "f79-5e28b29d38e93"
[1774289739] <<   accept-ranges: bytes
[1774289739] << [proxy->user] 200 OK (revalidated)
```

### В. Черный список (2 балла)
Прокси-сервер отслеживает страницы и не пускает на те, которые попадают в черный список. Вместо
этого прокси-сервер отправляет предупреждение, что страница заблокирована. Список доменов
и/или URL-адресов для блокировки по черному списку задается в **конфигурационном файле**.

Приложите скрины или логи запроса из черного списка.

#### Демонстрация работы
Добавил neverssl.com в черный список
```
[1774292691] >> [user->proxy] GET /neverssl.com
[1774292691] >>   Host: localhost:8000
[1774292691] >>   User-Agent: curl/8.7.1
[1774292691] >>   Accept: */*
[1774292691] << [proxy->user] 403 Forbidden
[1774292691] <<   body: http://neverssl.com is in the blacklist
```

## Wireshark. Работа с DNS
Для каждого задания в этой секции приложите скрин с подтверждением ваших ответов.

### А. Утилита nslookup (1 балл)

#### Вопросы
1. Выполните nslookup, чтобы получить IP-адрес какого-либо веб-сервера в Азии
   - <!-- todo -->
2. Выполните nslookup, чтобы определить авторитетные DNS-серверы для какого-либо университета в Европе
   - <!-- todo -->
3. Используя nslookup, найдите веб-сервер, имеющий несколько IP-адресов. Сколько IP-адресов имеет веб-сервер вашего учебного заведения?
   - <!-- todo -->
   - <!-- todo -->

### Б. DNS-трассировка www.ietf.org (3 балла)

#### Подготовка
1. Используйте ipconfig для очистки кэша DNS на вашем компьютере.
2. Откройте браузер и очистите его кэш (для Chrome можете использовать сочетание клавиш
   CTRL+Shift+Del).
3. Запустите Wireshark и введите `ip.addr == ваш_IP_адрес` в строке фильтра, где значение
   ваш_IP_адрес вы можете получить, используя утилиту ipconfig. Данный фильтр позволит
   нам отбросить все пакеты, не относящиеся к вашему хосту. Запустите процесс захвата пакетов в Wireshark.
4. Зайдите на страницу www.ietf.org в браузере.
5. Остановите захват пакетов.

#### Вопросы
1. Найдите DNS-запрос и ответ на него. С использованием какого транспортного протокола
   они отправлены?
   - <!-- todo -->
2. Какой порт назначения у запроса DNS?
   - <!-- todo -->
3. На какой IP-адрес отправлен DNS-запрос? Используйте ipconfig для определения IP-адреса
   вашего локального DNS-сервера. Одинаковы ли эти два адреса?
   - <!-- todo -->
   - <!-- todo -->
4. Проанализируйте сообщение-запрос DNS. Запись какого типа запрашивается? Содержатся
   ли в запросе какие-нибудь «ответы»?
   - <!-- todo -->
   - <!-- todo -->
5. Проанализируйте ответное сообщение DNS. Сколько в нем «ответов»? Что содержится в
   каждом?
   - <!-- todo -->
   - <!-- todo -->
6. Посмотрите на последующий TCP-пакет с флагом SYN, отправленный вашим компьютером.
   Соответствует ли IP-адрес назначения пакета с SYN одному из адресов, приведенных в
   ответном сообщении DNS?
   - <!-- todo -->
7. Веб-страница содержит изображения. Выполняет ли хост новые запросы DNS перед
   загрузкой этих изображений?
   - <!-- todo -->

### В. DNS-трассировка www.spbu.ru (2 балла)

#### Подготовка
1. Запустите захват пакетов с тем же фильтром `ip.addr == ваш_IP_адрес`
2. Выполните команду nslookup для сервера www.spbu.ru
3. Остановите захват
4. Вы увидите несколько пар запрос-ответ DNS. Найдите последнюю пару, все вопросы будут относиться к ней
   
#### Вопросы
1. Каков порт назначения в запросе DNS? Какой порт источника в DNS-ответе?
   - <!-- todo -->
   - <!-- todo -->
2. На какой IP-адрес отправлен DNS-запрос? Совпадает ли он с адресом локального DNS-сервера, установленного по умолчанию?
   - <!-- todo -->
   - <!-- todo -->
3. Проанализируйте сообщение-запрос DNS. Запись какого типа запрашивается? Содержатся
   ли в запросе какие-нибудь «ответы»?
   - <!-- todo -->
   - <!-- todo -->
4. Проанализируйте ответное сообщение DNS. Сколько в нем «ответов»? Что содержится в каждом?
   - <!-- todo -->
   - <!-- todo -->

### Г. DNS-трассировка nslookup –type=NS (1 балл)
Повторите все шаги по предварительной подготовке из Задания B, но теперь для команды `nslookup –type=NS spbu.ru`

#### Вопросы
1. На какой IP-адрес отправлен DNS-запрос? Совпадает ли он с адресом локального DNS-сервера, установленного по умолчанию?
   - <!-- todo -->
   - <!-- todo -->
2. Проанализируйте сообщение-запрос DNS. Запись какого типа запрашивается? Содержатся ли в запросе какие-нибудь «ответы»?
   - <!-- todo -->
   - <!-- todo -->
3. Проанализируйте ответное сообщение DNS. Имена каких DNS-серверов университета в
   нем содержатся? А есть ли их адреса в этом ответе?
   - <!-- todo -->
   - <!-- todo -->

### Д. DNS-трассировка nslookup www.spbu.ru ns2.pu.ru (1 балл)
Снова повторите все шаги по предварительной подготовке из Задания B, но теперь для команды `nslookup www.spbu.ru ns2.pu.ru`.
Запись `nslookup host_name dns_server` означает, что запрос на разрешение доменного имени `host_name` пойдёт к `dns_server`.
Если параметр `dns_server` не задан, то запрос идёт к DNS-серверу по умолчанию (например, к локальному).

#### Вопросы
1. На какой IP-адрес отправлен DNS-запрос? Совпадает ли он с адресом локального DNS-сервера, установленного по умолчанию? 
   Если нет, то какому хосту он принадлежит?
   - <!-- todo -->
   - <!-- todo -->
2. Проанализируйте сообщение-запрос DNS. Запись какого типа запрашивается? Содержатся
   ли в запросе какие-нибудь «ответы»?
   - <!-- todo -->
   - <!-- todo -->
3. Проанализируйте ответное сообщение DNS. Сколько в нем «ответов»? Что содержится в
   каждом?
   - <!-- todo -->
   - <!-- todo -->

### Е. Сервисы whois (2 балла)
1. Что такое база данных whois?
   - <!-- todo -->
2. Используя различные сервисы whois в Интернете, получите имена любых двух DNS-серверов. 
   Какие сервисы вы при этом использовали?
   - <!-- todo -->
   - <!-- todo -->
3. Используйте команду nslookup на локальном хосте, чтобы послать запросы трем конкретным
   серверам DNS (по аналогии с Заданием Д): вашему локальному серверу DNS и двум DNS-серверам,
   найденным в предыдущей части.
   - <!-- todo -->
