port module Boiding exposing (..)

import Browser
import Dict
import Domain.Flock as Flock
import Domain.Team as Team exposing (Name, team)
import Html
import Html.Attributes as Attribute
import Json.Decode exposing (decodeString, errorToString)
import Set
import Simulation exposing (Simulation, teamsOf)


main : Program Flags Model Message
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


init : Flags -> ( Model, Cmd Message )
init flags =
    let
        data =
            [ ( team "red-bergen-crab" True Flock.empty, True )
            , ( team "yellow-nijmegen-whale" False Flock.empty, True )
            , ( team "blue-ibiza-flamingo" False Flock.empty, False )
            ]

        teams =
            data
                |> List.map Tuple.first
                |> List.map (\t -> ( Team.nameOf t, t ))
                |> Dict.fromList

        visibleTeams =
            data
                |> List.map (Tuple.mapFirst Team.nameOf)
                |> Dict.fromList
    in
    ( { simulation = { teams = teams }
      , errorMessage = Nothing
      , visibleTeams = visibleTeams
      , hoverOver = Nothing
      , flags = flags
      }
    , Cmd.none
    )


type alias Flags =
    { size : Int }


type alias Model =
    { simulation : Simulation
    , errorMessage : Maybe String
    , visibleTeams : Dict.Dict String Bool
    , hoverOver : Maybe String
    , flags : Flags
    }


type Message
    = Update String
    | Spawn Name
    | ViewTeam Name Bool
    | Hover (Maybe Name)


update : Message -> Model -> ( Model, Cmd Message )
update message model =
    case message of
        Update updateMessage ->
            let
                nextModel =
                    case decodeString Simulation.decode updateMessage of
                        Ok simulation ->
                            let
                                existing =
                                    model.visibleTeams
                                        |> Dict.keys
                                        |> Set.fromList

                                fresh =
                                    simulation
                                        |> teamsOf
                                        |> Dict.keys
                                        |> Set.fromList

                                old =
                                    Set.diff existing fresh

                                new =
                                    Set.diff fresh existing

                                visibleTeams =
                                    model.visibleTeams
                                        |> (\map -> Set.foldl Dict.remove map old)
                                        |> (\map -> Set.foldl (\name -> Dict.insert name True) map new)
                            in
                            { model | simulation = simulation, visibleTeams = visibleTeams }

                        Err error ->
                            { model | errorMessage = Just (errorToString error) }
            in
            ( nextModel, Cmd.none )

        Spawn teamName ->
            ( model, spawn teamName )

        ViewTeam teamName showTeam ->
            let
                visibleTeams =
                    model.visibleTeams
                        |> Dict.insert teamName showTeam

                nextModel =
                    { model | visibleTeams = visibleTeams }
            in
            ( nextModel, Cmd.none )

        Hover hoverOver ->
            let
                nextModel =
                    { model | hoverOver = hoverOver }
            in
            ( nextModel, Cmd.none )


view : Model -> Html.Html Message
view model =
    let
        teams =
            model.simulation
                |> .teams
                |> Dict.values
                |> List.map (Team.view Spawn ViewTeam Hover model.visibleTeams model.hoverOver)

        errorMessage =
            Maybe.withDefault "" model.errorMessage
    in
    Html.div []
        [ Html.span [ Attribute.class "error" ] [ Html.text errorMessage ]
        , Html.div [ Attribute.class "teams" ] teams
        , Html.div [ Attribute.class "flocks" ] [ Simulation.view model.flags.size model.visibleTeams model.hoverOver model.simulation ]
        ]


port updateTeams : (String -> msg) -> Sub msg


port spawn : String -> Cmd msg


subscriptions : Model -> Sub Message
subscriptions _ =
    updateTeams Update
