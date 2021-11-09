module Simulation exposing (Simulation, decode, teamsOf, view)

import Dict exposing (Dict)
import Domain.Flock as Flock
import Domain.Team as Team exposing (Name, Team)
import Json.Decode as Decode exposing (Decoder, dict)
import Json.Decode.Pipeline exposing (required)
import Svg
import Svg.Attributes exposing (fill, height, stroke, strokeWidth, viewBox, width)


type alias Simulation =
    { teams : Dict Name Team
    }


teamsOf : Simulation -> Dict Name Team
teamsOf =
    .teams


view : Int -> Dict Name Bool -> Maybe Name -> Simulation -> Svg.Svg msg
view size visibleTeams attention simulation =
    let
        s =
            String.fromInt size

        shouldView team =
            visibleTeams
                |> Dict.get (Team.nameOf team)
                |> Maybe.withDefault True

        flocks =
            simulation
                |> .teams
                |> Dict.values
                |> List.filter shouldView
                |> List.map (viewFlockOf attention)
    in
    Svg.svg [ width s, height s, viewBox "0 0 1 1" ]
        [ Svg.g [ fill "white", stroke "black", strokeWidth "0.001" ] flocks
        ]


viewFlockOf : Maybe Name -> Team -> Svg.Svg msg
viewFlockOf attention team =
    let
        highlightColor sentinal =
            if sentinal then
                Just "blue"

            else
                Nothing

        stroke_color =
            attention
                |> Maybe.map (\name -> name == Team.nameOf team)
                |> Maybe.andThen highlightColor
                |> Maybe.withDefault "black"

        fill_color =
            team
                |> Team.nameOf
                |> String.split "-"
                |> List.head
                |> Maybe.withDefault "white"

        boids =
            team
                |> Team.flockOf
                |> Flock.view
    in
    Svg.g [ fill fill_color, stroke stroke_color ] boids


decode : Decoder Simulation
decode =
    Decode.succeed Simulation
        |> required "teams" (dict Team.decode)
