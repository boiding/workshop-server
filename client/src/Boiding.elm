port module Boiding exposing (..)

import Browser
import Dict
import Domain exposing (Team, Teams, decodeTeams, viewTeam)
import Html
import Html.Attributes as Attribute
import Json.Encode exposing (Value)
import Json.Decode exposing (decodeString, decodeValue, errorToString)


main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


init : () -> ( Model, Cmd Message )
init _ =
    let
        teams =
            Dict.empty
                |> Dict.insert "red-bergen-crab" { name = "red-bergen-crab", connected = True }
                |> Dict.insert "yellow-nijmegen-whale" { name = "yellow-nijmegen-whale", connected = False }
    in
    ( { team_repository = { teams = teams }
      , error_message = Nothing
      }
    , Cmd.none
    )


type alias Model =
    { team_repository : Teams
    , error_message : Maybe String
    }


type Message
    = Update String


update : Message -> Model -> ( Model, Cmd Message )
update message model =
    case message of
        Update updateMessage ->
            let
                next_model =
                    case decodeString decodeTeams updateMessage of
                        Ok teams ->
                            { model | team_repository = teams }

                        Err error ->
                            { model | error_message = Just (errorToString error) }
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


port updateTeams : (String -> msg) -> Sub msg

subscriptions : Model -> Sub Message
subscriptions model =
    updateTeams Update 