# O co jde

Chatovací aplikace. Jeden server a více klientů. Klienti si přes server posílají zprávy, obrázky a soubory.

# Jak spustit

**Server:**
```
cd hw10\server
cd cargo run -- -s 127.0.0.1 -p 8080
```

**Client:**
```
cd hw10\client
cd cargo run -- -s 127.0.0.1 -p 8080
```

# Design 

- neblokující
- čte z TCP streamu s timeoutem
- používá minimum vláken
- periodicky prochází všechny klienty a snaží se z nich číst

# Otazníky, podivnosti

## Čtení s timeoutem

Na serveru se potenciálně sleze více klientů. Nechci pro každýho vytvářet vlákno a v něm mít blokující `read`. Důvody:
- neškáluje to, vlákna nejsou zadarmo
- tipuju, že neexistuje nějaký pěkný zavření spojení než násilně zabít thread (protože na stream se kvůli ownershipu dostanu jen z toho threadu)

Proto se pokouším číst s timeoutem. Původně jsem si myslel, že použiju `TcpStream.peek` ale ten je blokující. Pokud do streamu ještě nikdo nezapsal, visí.
Implementace je tedy takhle škaredá:
```rust
pub fn receive(stream: &mut TcpStream) -> Result<Option<Message>, Box<dyn Error>> {
    // store original timeout so that it can be set later
    let timeout_original = stream.read_timeout().unwrap_or(None);

    // short timeout just to simulate peek
    stream.set_read_timeout(Some(STREAM_READ_TIMEOUT))?;

    // read size of sent data (first 4 bytes); trying to read with short timeout
    let data_len = {
        let mut len_bytes = [0u8; 4];
        let read_exact_result = stream.read_exact(&mut len_bytes);
        // set timeout back to original value
        stream.set_read_timeout(timeout_original)?;

        // we got the data length or timeouted - handle that
        match read_exact_result {
            Ok(_) => u32::from_be_bytes(len_bytes) as usize,
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::TimedOut => return Ok(None),     //timeout
                    std::io::ErrorKind::Interrupted => return Ok(None),     //timeout   - takhle je to v dokumentaci read_exact; ale ve skutecnosti hazi TimedOut
                    std::io::ErrorKind::UnexpectedEof => return Ok(None),   //client disconnected
                    _ => { println!("{:?}, kind: {}", e, e.kind()); return Err(Box::new(e)) }
                }
            },
        }
    };
    // lets read the rest, there should be the data with given length
    let message = {
        let mut buffer =  vec![0u8; data_len];
        stream.read_exact(&mut buffer)?;
        Message::deserialize(&buffer)?
    };
    Ok(Some(message))
}
```

Co mi nesedí:

1. pokud klient bude posílat dostatečně pomalu, celý se mi to rozdrbe. Klient může posílat data tak pomalu, že se nevleze do timeoutu. Tj. tady bych zase musel dát timeout na "nekonečno", ať to funguje správně. A jsem zpátky u blokujícího volání.

1. `Result<Option<_>, Box<dyn Error>>` - je tohle normální? Je to podivně škaredý, ale funkční.

1. Jak správně skombinovat více errorů dohromady? Mám funkci (viz sample), kde se může stát `std::io::Error` (setování timeoutu, timeout při čtení ze streamu), ale taky `bincode::Error` při deserializaci. Nevím, jak se toto řeší správně. 

## Uchovávání connection

Aktuálně (podle prezentace) mám takto:
```rust
struct ConnectedClients {
    clients: HashMap<SocketAddr, TcpStream>
}
```

Ale původně jsem měl jako:
```rust
struct ConnectedClients<'a> {
    clients: Vec<TcpStream>
    // pripadne hratky s &mut
    //clients: Vec<&'a mut TcpStream>
}
```

Tady jsem dost bojoval s ownershipem. Měl jsem totiž tento kód: 
```rust

fn read_messages_from_clients<'t>(clients: &'t mut ConnectedClients) -> Vec<(Message, &'t mut TcpStream)> {
    let mut received = Vec::new();
    for client in &mut clients.clients {
        // simplified a lot; more here: https://github.com/stej/rstnpc/commit/d340e8f31b72dcedff0ec70ff6e306d295f39b2e
        let msg = read_message(client);
        received.push((m, client))
    }
    received
}

let mut clients = ConnectedClients::new();

for stream in listener.incoming() {
    accept_connections(&mut clients, stream);
    let incomming_messages = read_messages_from_clients(&mut clients);

    // problem here, because clients were mut borrowed twice
    // the reason probably is that the TcpStream objects are stored in (*) clients, (*) incomming_messages

    broadcast_messages(&mut clients, incomming_messages);
    std::thread::sleep(Duration::from_millis(10));
}
```

Dobrej workaround tedy vypadá, že owner je `HashMap` a když chci se jen odkázat do ní, předávám si to pomocí klíčů `SocketAddr`, který jsou `Copy`.