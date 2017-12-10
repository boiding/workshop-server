module Boiding exposing (..)

import Html
import Html.Attributes as Attribute
import Dict
import WebSocket
import Domain exposing (Teams, Team, viewTeam)


main =
    Html.programWithFlags
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


type alias Flags =
    { socket_address : String
    }


init : Flags -> ( Model, Cmd Message )
init flags =
    let
        teams =
            Dict.empty
                |> Dict.insert "red-bergen-crab" { name = "red-bergen-crab", connected = True }
                |> Dict.insert "yellow-nijmegen-whale" { name = "yellow-nijmegen-whale", connected = False }
    in
        ( { socket_address = flags.socket_address
          , team_repository = { teams = teams }
          , message = Nothing
          }
        , Cmd.none
        )


type alias Model =
    { team_repository : Teams
    , socket_address : String
    , message : Maybe String
    }


type Message
    = Update String


update : Message -> Model -> ( Model, Cmd Message )
update message model =
    case message of
        Update message ->
            ( { model | message = Just message }, Cmd.none )


view : Model -> Html.Html Message
view model =
    let
        teams =
            model.team_repository
                |> .teams
                |> Dict.values
                |> List.map viewTeam

        message =
            Maybe.withDefault "no message" model.message
    in
        Html.div []
            [ Html.span [] [ Html.text message ]
            , Html.div [ Attribute.class "teams" ] teams
            ]


subscriptions : Model -> Sub Message
subscriptions model =
    WebSocket.listen model.socket_address Update
