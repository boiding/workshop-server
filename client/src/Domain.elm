module Domain exposing (Team, Teams, decodeTeams, viewTeam, viewFlocks)

import Dict
import Html
import Html.Attributes as Attribute
import Html.Events as Event
import Json.Decode exposing (Decoder, bool, dict, float, string, succeed)
import Json.Decode.Pipeline exposing (required)
import Svg
import Svg.Attributes exposing (width, height, viewBox, cx, cy, r, fill, stroke, strokeWidth) 


type alias Teams =
    { teams : Dict.Dict String Team
    }


type alias Team =
    { name : String
    , connected : Bool
    , flock : Flock
    }

type alias Flock =
    { boids : Dict.Dict String Boid }

type alias Boid =
    { x: Float
    , y: Float
    , heading: Float
    , speed: Float}

viewTeam : (String -> msg) -> Team -> Html.Html msg
viewTeam messageFor team =
    Html.div
        [ Attribute.classList
            [ ( "team", True )
            , ( "disconnected", not team.connected )
            , ( "connected", team.connected )
            ]
        ]
        [ Html.span [ Attribute.class "connection-status" ] []
        , Html.span [ Attribute.class "name" ] [ Html.text team.name ]
        , Html.button [ Event.onClick <| messageFor team.name ] [ Html.text "+" ]
        ]

viewFlocks : Teams -> Svg.Svg msg
viewFlocks teams =
    let
        flocks =
            teams
            |> .teams
            |> Dict.values
            |> List.map viewFlockOf
    in
    Svg.svg [ width "640", height "640", viewBox "0 0 1 1" ] [
        Svg.g [ fill "white", stroke "black", strokeWidth "0.001"] flocks
    ]

viewFlockOf : Team -> Svg.Svg msg
viewFlockOf team =
    let
        boids =
            team
            |> .flock
            |> .boids
            |> Dict.values
            |> List.map viewBoid
    in
    Svg.g [] boids

viewBoid : Boid -> Svg.Svg msg
viewBoid boid =
    Svg.circle [ cx <| String.fromFloat boid.x, cy <| String.fromFloat boid.y, r "0.01"] []


decodeTeams : Decoder Teams
decodeTeams =
    succeed Teams
        |> required "teams" (dict decodeTeam)


decodeTeam : Decoder Team
decodeTeam =
    succeed Team
        |> required "name" string
        |> required "connected" bool
        |> required "flock" decodeFlock

decodeFlock : Decoder Flock
decodeFlock =
    succeed Flock
        |> required "boids" (dict decodeBoid)

decodeBoid : Decoder Boid
decodeBoid = 
    succeed Boid
        |> required "x" float
        |> required "y" float
        |> required "heading" float
        |> required "speed" float