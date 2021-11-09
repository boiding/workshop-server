module Domain.Flock exposing (Flock, decode, empty, view)

import Dict exposing (Dict)
import Domain.Boid as Boid exposing (Boid)
import Json.Decode as Decode exposing (Decoder, dict)
import Json.Decode.Pipeline exposing (required)
import Svg exposing (Svg)


type Flock
    = Flock { boids : Dict String Boid }


flock : Dict String Boid -> Flock
flock boids =
    Flock { boids = boids }


empty : Flock
empty =
    flock Dict.empty


view : Flock -> List (Svg msg)
view (Flock f) =
    f
        |> .boids
        |> Dict.values
        |> List.map Boid.view


decode : Decoder Flock
decode =
    Decode.succeed flock
        |> required "boids" (dict Boid.decode)
