# Appunti iniziali

JSON DENORMALIZZATO CHE CI ARRIVA ALL'INIZIO
{
    orderId --> lines,
    customer--> email, name
    lines --> orderLineId, seatId, seatLabel, sectorId, sector, price 
    price --> priceAmount, priceCurrency
    eventId --> eventTitle, 
    sessionId --> venueName, venueCity, startsAt 
    sectorId --> sectorName
    seatId --> seatLabel, seatRow, seatNumber
}


EMAIL (customerEmail):
- QRCODE
- INFO SU EVENTO: eventTitle, startAt
- LOCATION: venueCity, venueName
- BIGLIETTO: sectorName, seatLabel, seatRow, seatNumber, prezzo



QrCode:
    - ticketId

----
# Schema logico

TICKET_META(
    ticket_meta_id                UUID NOT NULL     PK,
    order_id                       NOT NULL UNIQUE
    event_id
    event_title
    session_id
    session_starts_at
    venue_name
    venue_city

    customer_id
    customer_email
    customer_name
)

TICKET(
    ticket_id                     UUID     PK,
    meta_id         NOT_NULL      UUID     FK --> TICKET_META.ticket_meta_id,
    order_line_id
    sector_id
    sector_name
    seat_id
    seat_label
    seat_row
    seat_number
    issued_at 
    status ENUM(SCANNED, UNSCANNED, REVOKED)
)



TICKET_SCAN(
    scan_id                     PK,
    ticket_id                   FK --> TICKET.ticketId
    result ENUM(ADMITTED, NOT_ADMITTED)
    scanned_at  
)

## TODO
- Nella risposta alla post /api/v1/tickets/{ticketId}/scans dobbiamo rimuovere dalla response l'identificativo del gate che non è stato modellato e dipende dalla venue gestita dal microservizio catalog (e non è di nostro interesse)

- Credential_encrypted genera solo confusione. Lo toglieremo.
- Gestire il flusso del QRCODE - potrebbe essere un'immagine generata dal ticket_id e ne facciamo lo storage con RustFS (bucket)

# NOTE
- [x] L'endpoint POST che invia un ordine da sales a tickets non serve: tickets prende gli ordini dalla coda MQ.
- [] TICKET_META.order_id
- [x] Non mettiamo un vincolo di un solo posto per sessione in tickets perchè lo gestisce Sales (non dobbiamo fare modifiche allo schema logico scritto sopra)
- La vendita di biglietti avviene per una singola sessione (non posso comprare biglietti di sessioni diverse - stessa cosa per evento)
- `UNIQUE (order_id, seat_id)` e `UNIQUE (order_id, order_line_id)` sono gestiti da sales --> qui non servono

# OVERVIEW
- Avremo 2 feature: uno che genera i biglietti, uno che scansiona i biglietti + send_notification
- Inserimento delle porte degli adapter al database per i repository - il servizio ha all'interno un repository. Questo repository sa già su che porta ascoltare. Nell'app_state dobbiamo completiamo la comunicazione inserendo l'adapter a postgres. 


- Mapping da DTO a Record -- Sia per Order che OrderLine che per Ticket