port module Boiding exposing (..)

import Browser
import Dict
import Domain exposing (Team, Teams, decodeTeams, viewFlocks, viewTeam)
import Html
import Html.Attributes as Attribute
import Json.Decode exposing (decodeString, decodeValue, errorToString)
import Json.Encode exposing (Value)


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
                |> Dict.insert "red-bergen-crab" { name = "red-bergen-crab", connected = True, flock = { boids = Dict.empty } }
                |> Dict.insert "yellow-nijmegen-whale" { name = "yellow-nijmegen-whale", connected = False, flock = { boids = Dict.empty } }
                |> Dict.insert "blue-ibiza-flamingo" { name = "blue-ibiza-flamingo", connected = False, flock = { boids = Dict.empty } }

        show_team =
            Dict.empty
                |> Dict.insert "red-bergen-crab" True
                |> Dict.insert "yellow-nijmegen-whale" True
                |> Dict.insert "blue-ibiza-flamingo" False
    in
    ( { team_repository = { teams = teams }
      , error_message = Nothing
      , show_team = show_team
      }
    , Cmd.none
    )


type alias Model =
    { team_repository : Teams
    , error_message : Maybe String
    , show_team : Dict.Dict String Bool
    }


type Message
    = Update String
    | Spawn String
    | ViewTeam String Bool


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

        Spawn team_name ->
            ( model, spawn team_name )

        ViewTeam team_name show_it ->
            let
                show_team =
                    model.show_team
                        |> Dict.insert team_name show_it

                next_model =
                    { model | show_team = show_team }
            in
            ( next_model, Cmd.none )


view : Model -> Html.Html Message
view model =
    let
        teams =
            model.team_repository
                |> .teams
                |> Dict.values
                |> List.map (viewTeam Spawn ViewTeam model.show_team)

        error_message =
            Maybe.withDefault "" model.error_message
    in
    Html.div []
        [ Html.span [ Attribute.class "error" ] [ Html.text error_message ]
        , Html.div [ Attribute.class "teams" ] teams
        , Html.div [ Attribute.class "flocks" ] [ viewFlocks model.team_repository ]
        ]


port updateTeams : (String -> msg) -> Sub msg


port spawn : String -> Cmd msg


subscriptions : Model -> Sub Message
subscriptions model =
    updateTeams Update
