(function(document, Elm, socket_port){
    const socket_address = 'ws://127.0.0.1:' + socket_port;
    const container = document.getElementById('client-container');
    Elm.Boiding.embed(container, {
        'socket_address': socket_address
    });
})(document, Elm, 3435);
