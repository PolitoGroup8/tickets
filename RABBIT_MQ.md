Con il docker-compose del servizio RabbitMQ attivo, andare su `http://localhost:15672/`.
Username e password sono nel nostro `.env`.

Vai su `Queues and Streams`, seleziona la coda (io ho messo **"tickets.order.paid"**), dopodichè al fondo **Publish message**
Inserisci un payload come 
```json
{
  "type": "OrderPaid",
  "version": 1,
  "data": {
    "orderId": "ordine-demo-2",
    "customerId": "cliente-demo-1",
    "lines": [
      {
        "orderLineId": "demo-line-3",
        "seatId": "seat-16",
        "seatLabel": "E16",
        "sectorId": "s1",
        "sector": "platea"
      },
      {
        "orderLineId": "demo-line-4",
        "seatId": "seat-17",
        "seatLabel": "E17",
        "sectorId": "s1",
        "sector": "platea"
      }
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