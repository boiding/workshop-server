module Domain.Team exposing (Name, Team, decode, flockOf, nameOf, team, view)

import Dict exposing (Dict)
import Domain.Flock as Flock exposing (Flock)
import Html exposing (Html)
import Html.Attributes as Attribute
import Html.Events as Event
import Json.Decode as Decode exposing (Decoder, bool, string)
import Json.Decode.Pipeline exposing (required)

type Team
    = Team
        { name : Name
        , connected : Bool
        , flock : Flock
        }


type alias Name =
    String


team : Name -> Bool -> Flock -> Team
team name connected flock =
    Team
        { name = name
        , connected = connected
        , flock = flock
        }


nameOf : Team -> Name
nameOf (Team aTeam) =
    aTeam.name


flockOf : Team -> Flock
flockOf (Team aTeam) =
    aTeam.flock


view :
    (Name -> msg)
    -> (Name -> Bool -> msg)
    -> (Maybe Name -> msg)
    -> Dict Name Bool
    -> Maybe Name
    -> Team
    -> Html msg
view addBoidsFor show focusOn visibleTeams attention (Team aTeam) =
    let
        checked =
            visibleTeams
                |> Dict.get aTeam.name
                |> Maybe.withDefault True
    in
    Html.div
        [ Event.onMouseEnter <| focusOn (Just aTeam.name)
        , Event.onMouseLeave <| focusOn Nothing
        , Attribute.classList
            [ ( "team", True )
            , ( "disconnected", not aTeam.connected )
            , ( "connected", aTeam.connected )
            , ( "attention", attention |> Maybe.map (\name -> name == aTeam.name) |> Maybe.withDefault False )
            ]
        ]
        [ Html.input [ Event.onCheck <| show aTeam.name, Attribute.type_ "checkbox", Attribute.checked checked ] []
        , Html.span [ Attribute.class "connection-status" ] []
        , Html.span [ Attribute.class "name" ] [ Html.text aTeam.name ]
        , Html.button [ Event.onClick <| addBoidsFor aTeam.name ] [ Html.text "+" ]
        ]


decode : Decoder Team
decode =
    Decode.succeed team
        |> required "name" string
        |> required "connected" bool
        |> required "flock" Flock.decode
