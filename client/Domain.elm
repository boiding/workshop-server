module Domain exposing (Teams, Team, viewTeam, decodeTeams)

import Json.Decode exposing (Decoder, dict, string, bool)
import Json.Decode.Pipeline exposing (decode, required)
import Html
import Html.Attributes as Attribute
import Dict


type alias Teams =
    { teams : Dict.Dict String Team
    }


type alias Team =
    { name : String
    , connected : Bool
    }


viewTeam : Team -> Html.Html msg
viewTeam team =
    Html.div
        [ Attribute.classList
            [ ( "team", True )
            , ( "disconnected", not team.connected )
            , ( "connected", team.connected )
            ]
        ]
        [ Html.span [ Attribute.class "connection-status" ] []
        , Html.span [ Attribute.class "name" ] [ Html.text team.name ]
        ]


decodeTeams : Decoder Teams
decodeTeams =
    decode Teams
        |> required "teams" (dict decodeTeam)


decodeTeam : Decoder Team
decodeTeam =
    decode Team
        |> required "name" string
        |> required "connected" bool