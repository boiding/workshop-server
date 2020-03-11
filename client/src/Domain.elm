module Domain exposing (Team, Teams, decodeTeams, viewFlocks, viewTeam)

import Dict
import Html
import Html.Attributes as Attribute
import Html.Events as Event
import Json.Decode exposing (Decoder, bool, dict, float, string, succeed)
import Json.Decode.Pipeline exposing (required)
import Svg
import Svg.Attributes exposing (cx, cy, fill, height, points, r, stroke, strokeWidth, viewBox, width)


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
    { x : Float
    , y : Float
    , heading : Float
    , speed : Float
    }


viewTeam : (String -> msg) -> (String -> Bool -> msg) -> (Maybe String -> msg) -> Dict.Dict String Bool -> Team -> Html.Html msg
viewTeam onPlus onCheck onHover show_team team =
    let
        checked =
            show_team
                |> Dict.get team.name
                |> Maybe.withDefault True
    in
    Html.div
        [ Event.onMouseEnter <| onHover (Just team.name)
        , Event.onMouseOut <| onHover Nothing
        , Attribute.classList
            [ ( "team", True )
            , ( "disconnected", not team.connected )
            , ( "connected", team.connected )
            ]
        ]
        [ Html.input [ Event.onCheck <| onCheck team.name, Attribute.type_ "checkbox", Attribute.checked checked ] []
        , Html.span [ Attribute.class "connection-status" ] []
        , Html.span [ Attribute.class "name" ] [ Html.text team.name ]
        , Html.button [ Event.onClick <| onPlus team.name ] [ Html.text "+" ]
        ]


viewFlocks : Dict.Dict String Bool -> Teams -> Svg.Svg msg
viewFlocks view_team teams =
    let
        should_view team =
            view_team
                |> Dict.get team.name
                |> Maybe.withDefault True

        flocks =
            teams
                |> .teams
                |> Dict.values
                |> List.filter should_view
                |> List.map viewFlockOf
    in
    Svg.svg [ width "640", height "640", viewBox "0 0 1 1" ]
        [ Svg.g [ fill "white", stroke "black", strokeWidth "0.001" ] flocks
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
    let
        r =
            0.01

        a =
            2 * pi / 3

        circlePoint angle =
            ( r * cos angle + boid.x, r * sin angle + boid.y )

        path =
            [ circlePoint boid.heading, circlePoint (boid.heading + a), ( boid.x, boid.y ), circlePoint (boid.heading - a) ]
                |> List.map (\( x, y ) -> String.fromFloat x ++ "," ++ String.fromFloat y)
                |> String.join " "
    in
    Svg.polygon [ points path ] []


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
