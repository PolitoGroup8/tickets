Con il docker-compose del servizio RabbitMQ attivo, andare su `http://localhost:15672/`.
Username e password sono nel nostro `.env`.

Vai su `Queues and Streams`, seleziona la coda (io ho messo **"tickets.order.paid"**), dopodichè al fondo **Publish message**
Inserisci un payload come 
```json
{
  "type": "OrderPaid",
  "version": 1,
  "data":{
    "orderId":"ordine-1",
    "customerId":"cliente-1",
    "customerEmail": "customer@example.test",
    "event": {
      "eventId": "evt_test",
      "sessionId": "ses_test",
      "eventTitle": "Concerto",
      "startsAt": "2026-12-10T20:00:00Z",
      "venueName": "Teatro",
      "venueCity": "Torino"
    },
    "lines":[
      { "orderLineId": "line-1", "seatId": "seat-14", "seatLabel": "E14", "sectorId": "s1", "sector": "platea" },
      { "orderLineId": "line-2", "seatId": "seat-15", "seatLabel": "E15", "sectorId": "s2", "sector": "piccionaia" }
    ]
  }
}
```


Controlla sul terminale dove gira cargo. Dovresti trovare qualcosa come:
```json
[
    NewTicket {
        order_line_id: "demo-line-3",
        seat_id: "seat-16",
        seat_label: "E16",
        sector_id: "s1",
        sector_name: "platea",
        status: Unscanned,
    },
    NewTicket {
        order_line_id: "demo-line-4",
        seat_id: "seat-17",
        seat_label: "E17",
        sector_id: "s1",
        sector_name: "platea",
        status: Unscanned,
    },
]
```