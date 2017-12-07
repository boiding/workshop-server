module Boiding exposing (..)

import Html
import Html.Attributes as Attribute
import Dict


main =
    Html.program
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


init : ( Model, Cmd Message )
init =
    let
        teams =
            Dict.empty
                |> Dict.insert "red-bergen-crab" { name = "red-bergen-crab", connected = True }
                |> Dict.insert "yellow-nijmegen-whale" { name = "yellow-nijmegen-whale", connected = False }
    in
        ( { team_repository = { teams = teams } }, Cmd.none )


type alias Model =
    { team_repository : Teams
    }


type alias Teams =
    { teams : Dict.Dict String Team
    }


type alias Team =
    { name : String
    , connected : Bool
    }


type Message
    = DoNothing


update : Message -> Model -> ( Model, Cmd Message )
update message model =
    case message of
        DoNothing ->
            ( model, Cmd.none )


view : Model -> Html.Html Message
view model =
    let
        teams =
            model.team_repository
                |> .teams
                |> Dict.values
                |> List.map viewTeam
    in
        Html.div [ Attribute.class "teams" ] teams


viewTeam : Team -> Html.Html Message
viewTeam team =
    Html.div
        [ Attribute.classList
            [ ( "team", True )
            , ( "disconnected", not team.connected )
            , ( "connected", team.connected )
            ]
        ]
        [ Html.span [] [ Html.text team.name ]
        ]


subscriptions : Model -> Sub Message
subscriptions model =
    Sub.none
