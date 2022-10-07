welcome = Welcome { $first-name } on { -app }!

state =
    .online = Online
    .offline = Offline
    .busy = Busy ({ $reason })
    .busy-for = Busy for { $hours ->
        [1] 1 hour
       *[other] { $hours } hours
    } ({ $reason })
