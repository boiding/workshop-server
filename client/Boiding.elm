module Boiding exposing (..)

import Json.Decode exposing (decodeString)
import Html
import Html.Attributes as Attribute
import Dict
import WebSocket
import Domain exposing (Teams, Team, viewTeam, decodeTeams)


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
          , error_message = Nothing
          }
        , Cmd.none
        )


type alias Model =
    { team_repository : Teams
    , socket_address : String
    , error_message : Maybe String
    }


type Message
    = Update String


update : Message -> Model -> ( Model, Cmd Message )
update message model =
    case message of
        Update update ->
            let
                next_model =
                    case decodeString decodeTeams update of
                        Ok teams ->
                            { model | team_repository = teams }

                        Err error ->
                            { model | error_message = Just (toString error) }
            in
                ( next_model, Cmd.none )


view : Model -> Html.Html Message
view model =
    let
        teams =
            model.team_repository
                |> .teams
                |> Dict.values
                |> List.map viewTeam

        error_message =
            Maybe.withDefault "" model.error_message
    in
        Html.div []
            [ Html.span [ Attribute.class "error" ] [ Html.text error_message ]
            , Html.div [ Attribute.class "teams" ] teams
            ]


subscriptions : Model -> Sub Message
subscriptions model =
    WebSocket.listen model.socket_address Update
