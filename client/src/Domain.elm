module Domain exposing (Team, Teams, decodeTeams, viewTeam)

import Dict
import Html
import Html.Attributes as Attribute
import Html.Events as Event
import Json.Decode exposing (Decoder, bool, dict, float, string, succeed)
import Json.Decode.Pipeline exposing (required)


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