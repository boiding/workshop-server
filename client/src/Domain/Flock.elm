module Domain.Flock exposing (Flock, decode, empty, view)

import Dict exposing (Dict)
import Domain.Boid as Boid exposing (Boid)
import Domain.Food as Food exposing (Food)
import Json.Decode as Decode exposing (Decoder, dict, list)
import Json.Decode.Pipeline exposing (required)
import Svg exposing (Svg)


type Flock
    = Flock
        { boids : Dict String Boid
        , food : List Food
        }


flock : Dict String Boid -> List Food -> Flock
flock boids food =
    Flock
        { boids = boids
        , food = food
        }


empty : Flock
empty =
    flock Dict.empty []


view : Flock -> List (Svg msg)
view (Flock f) =
    let
        food =
            f
                |> .food
                |> List.map Food.view

        boids =
            f
                |> .boids
                |> Dict.values
                |> List.map Boid.view
    in
    List.concat [ food, boids ]


decode : Decoder Flock
decode =
    Decode.succeed flock
        |> required "boids" (dict Boid.decode)
        |> required "food" (list Food.decode)
