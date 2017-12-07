module Boiding exposing (..)

import Html


main =
    Html.program
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


init : ( Model, Cmd Message )
init =
    ( { sentence = "Hello, World" }, Cmd.none )


type alias Model =
    { sentence : String
    }


type Message
    = Say String


update : Message -> Model -> ( Model, Cmd Message )
update message model =
    case message of
        Say sentence ->
            ( { model | sentence = sentence }, Cmd.none )


view : Model -> Html.Html Message
view model =
    Html.div []
        [ Html.span []
            [ Html.text model.sentence ]
        ]



subscriptions : Model -> Sub Message
subscriptions model =
    Sub.none
