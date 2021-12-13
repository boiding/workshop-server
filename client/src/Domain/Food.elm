module Domain.Food exposing (Food, decode, food, view)

import Json.Decode as Decode exposing (Decoder, float)
import Json.Decode.Pipeline exposing (required)
import Svg exposing (Svg)
import Svg.Attributes exposing (cx, cy, radius, transform)


type Food
    = Food Location


type alias Location =
    { x : Float, y : Float }


food : Float -> Float -> Food
food x y =
    Food { x = x, y = y }


view : Food -> Svg msg
view (Food location) =
    Svg.g []
        [ Svg.circle
            [ cx <| String.fromFloat location.x
            , cy <| String.fromFloat location.y
            , radius "0.01"
            ]
            []
        ]


decode : Decoder Food
decode =
    Decode.succeed food
        |> required "x" float
        |> required "y" float
