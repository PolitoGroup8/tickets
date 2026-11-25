# RabbitMQ in Tickets

## Prova manuale di `ORDER_PAID`

Per allinearsi al formato concordato con Sales, Tickets legge `"type": "ORDER_PAID"` nel JSON. Serde associa questo valore alla variante Rust `OrderMessage::OrderPaid`.

1. Avvia RabbitMQ e PostgreSQL. Dalla directory di Tickets esegui `cargo run` e attendi `Listening on tickets.order-paid` nel terminale.
2. Apri [RabbitMQ Management](http://localhost:15672/) e accedi con le credenziali del `.env` del broker.
3. Vai in **Queues and Streams**, apri `tickets.order-paid` e seleziona **Publish message**. Se compare il selettore **Protocol**, scegli **AMQP 0-9-1**. Usa **String** come codifica del payload e incolla il JSON qui sotto.

Usa un `orderId` nuovo per ogni prova manuale. La tabella `ticket_meta` accetta un solo record per ordine.

### Payload `ORDER_PAID`

```json
{
  "type": "ORDER_PAID",
  "version": 1,
  "data": {
    "orderId": "ordine-1",
    "customerId": "cliente-1",
    "customerEmail": "customer@example.test",
    "event": {
      "eventId": "evt_test",
      "sessionId": "ses_test",
      "eventTitle": "Concerto",
      "startsAt": "2026-12-10T20:00:00Z",
      "venueName": "Teatro",
      "venueCity": "Torino"
    },
    "lines": [
      {
        "seatId": "seat-14",
        "seatLabel": "E14",
        "sectorId": "s1",
        "sector": "platea"
      },
      {
        "seatId": "seat-15",
        "seatLabel": "E15",
        "sectorId": "s2",
        "sector": "piccionaia"
      }
    ]
  }
}
```

### Risultato atteso

Tickets salva due biglietti e li stampa nel terminale. I posti `E14` ed `E15` sono entrambi `Unscanned`. PostgreSQL assegna gli UUID durante il salvataggio, quindi quelli qui sotto sono solo esempi.

```text
[
    NewTicket {
        ticket_id: "b63fe671-a440-4f05-afbf-585dcc28a388",
        seat_id: "seat-14",
        seat_label: "E14",
        sector_id: "s1",
        sector_name: "platea",
        status: Unscanned,
    },
    NewTicket {
        ticket_id: "88037922-0f28-4fa6-aa5f-ccd2dd8dca16",
        seat_id: "seat-15",
        seat_label: "E15",
        sector_id: "s2",
        sector_name: "piccionaia",
        status: Unscanned,
    },
]
```

## Notifiche con i QR in allegato

Flusso concordato da implementare.

1. Dopo aver salvato i biglietti, Tickets genera un QR dalla credenziale di ingresso di ciascun biglietto e carica il PNG in un bucket privato su RustFS.
2. Quando tutti i PNG dell’ordine sono disponibili, Tickets pubblica un evento `TicketsIssued` su RabbitMQ con il tipo `TICKETS_ISSUED`.
3. Notifications scarica i PNG con le proprie credenziali di lettura e li allega a un’unica email per ordine.

Tickets ricava `qrObjectKey` dal `ticket_id` salvato in PostgreSQL, usando la regola `tickets/<ticketId>/qr.png`. Usa la stessa chiave per salvare il PNG su RustFS e per compilare `TICKETS_ISSUED`.

Notifications usa la chiave ricevuta per trovare il PNG nel bucket configurato, anche se deve ritentare l’invio dell’email.

### Payload `TICKETS_ISSUED`

L’esempio riprende l’ordine e i due biglietti della prova sopra.

```json
{
  "eventId": "85fb487d-28ce-4e47-a525-236fa564895a",
  "type": "TICKETS_ISSUED",
  "version": 1,
  "producer": "tickets",
  "occurredAt": "2026-11-25T17:00:00Z",
  "data": {
    "orderId": "ordine-1",
    "customerEmail": "customer@example.test",
    "tickets": [
      {
        "ticketId": "b63fe671-a440-4f05-afbf-585dcc28a388",
        "qrObjectKey": "tickets/b63fe671-a440-4f05-afbf-585dcc28a388/qr.png"
      },
      {
        "ticketId": "88037922-0f28-4fa6-aa5f-ccd2dd8dca16",
        "qrObjectKey": "tickets/88037922-0f28-4fa6-aa5f-ccd2dd8dca16/qr.png"
      }
    ]
  }
}
```

### Ritenti

Se un caricamento fallisce, Tickets ritenta usando la stessa chiave del file. Pubblica l’evento quando tutti i PNG sono pronti.

Se Tickets ritenta la pubblicazione, riusa lo stesso `eventId`. Notifications usa quell’ID per riconoscere la riconsegna e accodare la mail una sola volta.
