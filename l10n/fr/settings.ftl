status =
    .online = En ligne
    .offline = Hors ligne
    .busy = { $gender ->
        [male] Occupé
        [female] Occupée
       *[other] Non disponible
    } ({ $reason })
    .busy-for = { status.busy } [{ TIME($time) }]
