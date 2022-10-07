welcome = Bienvenue { $first-name } { $last-name } sur { -app(case: "lowercase") }.

state =
    .online = En ligne
    .offline = Hors ligne
    .busy = { $gender ->
        [male] Occupé
        [female] Occupée
       *[other] Non disponible
    } ({ $reason })
    .busy-for = { $gender ->
        [male] Occupé
        [female] Occupée
       *[other] Non disponible
    } pour { $hours ->
        [1] 1 heure
       *[other] { $hours } heures
    } ({ $reason })
