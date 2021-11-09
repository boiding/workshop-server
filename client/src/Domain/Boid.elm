module Domain.Boid exposing (Boid, decode, view)

import Json.Decode as Decode exposing (Decoder, float)
import Json.Decode.Pipeline exposing (required)
import Svg exposing (Svg)
import Svg.Attributes exposing (points)


type Boid
    = Boid
        { x : Float
        , y : Float
        , heading : Float
        , speed : Float
        }


boid : Float -> Float -> Float -> Float -> Boid
boid x y heading speed =
    Boid { x = x, y = y, heading = heading, speed = speed }


view : Boid -> Svg msg
view (Boid b) =
    let
        r =
            0.01

        a =
            2 * pi / 3

        circlePoint angle =
            ( r * cos angle + b.x, r * sin angle + b.y )

        path =
            [ circlePoint b.heading, circlePoint (b.heading + a), ( b.x, b.y ), circlePoint (b.heading - a) ]
                |> List.map (\( x, y ) -> String.fromFloat x ++ "," ++ String.fromFloat y)
                |> String.join " "
    in
    Svg.polygon [ points path ] []


decode : Decoder Boid
decode =
    Decode.succeed boid
        |> required "x" float
        |> required "y" float
        |> required "heading" float
        |> required "speed" float
