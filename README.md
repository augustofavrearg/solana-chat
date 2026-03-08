# Project Chat Smart Contract

A Solana Anchor program that adds decentralized discussion threads, comments, replies, and reactions to external posts.

Un programa Solana Anchor que agrega hilos de discusión descentralizados, comentarios, respuestas y reacciones a publicaciones externas.

## Features

- Creates one discussion thread PDA per external post.
- Crea una PDA de hilo de discusión por cada publicación externa.

- Supports top-level comments and nested replies.
- Soporta comentarios de primer nivel y respuestas anidadas.

- Supports per-user reactions on comments using deterministic PDAs.
- Soporta reacciones por usuario en comentarios usando PDAs determinísticas.

- Includes moderation controls: comment deletion, thread locking, and admin overrides.
- Incluye controles de moderación: borrado de comentarios, bloqueo de hilos y acciones administrativas.

- Supports runtime governance via config updates (`paused`, eligibility mode, limits).
- Soporta gobernanza en tiempo de ejecución mediante actualización de configuración (`paused`, modo de elegibilidad, límites).

## Program Architecture

The program stores global settings in `Config`, thread metadata in `Thread`, message content in `Comment`, and user reactions in `Reaction`.

El programa guarda la configuración global en `Config`, metadatos del hilo en `Thread`, contenido de mensajes en `Comment` y reacciones de usuario en `Reaction`.

Main PDA seeds are: `config`, `thread + source_program_id + post_account`, `comment + thread + comment_id`, and `reaction + comment + user + reaction_kind`.

Las seeds principales de PDA son: `config`, `thread + source_program_id + post_account`, `comment + thread + comment_id` y `reaction + comment + user + reaction_kind`.

## Instructions

- `initialize_config`: initializes global configuration and sets the super admin.
- `initialize_config`: inicializa la configuración global y define el super admin.

- `update_config`: updates pause state, eligibility mode, and limits.
- `update_config`: actualiza estado de pausa, modo de elegibilidad y límites.

- `create_thread_for_post`: creates a thread linked to an external post after validating `PostMetaStandard`.
- `create_thread_for_post`: crea un hilo vinculado a una publicación externa tras validar `PostMetaStandard`.

- `create_comment`: creates a top-level comment after validating user eligibility.
- `create_comment`: crea un comentario de primer nivel tras validar elegibilidad del usuario.

- `reply_comment`: creates a reply linked to a parent comment and root comment.
- `reply_comment`: crea una respuesta vinculada al comentario padre y al comentario raíz.

- `edit_comment`: lets only the original author update the comment body.
- `edit_comment`: permite solo al autor original actualizar el cuerpo del comentario.

- `delete_comment`: allows deletion by comment author, post author, or super admin (soft delete to `[deleted]`).
- `delete_comment`: permite borrar por autor del comentario, autor del post o super admin (borrado lógico a `[deleted]`).

- `set_thread_lock`: allows post author or super admin to lock/unlock a thread.
- `set_thread_lock`: permite al autor del post o super admin bloquear/desbloquear un hilo.

- `add_reaction`: creates a reaction account and increments comment reaction counter.
- `add_reaction`: crea una cuenta de reacción e incrementa el contador de reacciones del comentario.

- `remove_reaction`: removes caller-owned reaction and decrements counter.
- `remove_reaction`: elimina la reacción del usuario que llama y decrementa el contador.

- `admin_remove_reaction`: allows post author or super admin to remove any reaction.
- `admin_remove_reaction`: permite al autor del post o super admin eliminar cualquier reacción.

## Security Model

Every write path validates signer presence and rejects unauthorized actors for privileged operations.

Cada ruta de escritura valida la firma requerida y rechaza actores no autorizados en operaciones privilegiadas.

The program validates account owners for external metadata (`post_meta`, `user_meta`) and enforces canonical PDA derivation.

El programa valida propietarios de cuentas para metadatos externos (`post_meta`, `user_meta`) y exige derivación canónica de PDAs.

The Instructions sysvar is validated and introspected to harden instruction context checks.

Se valida e inspecciona el sysvar de Instructions para reforzar los chequeos del contexto de instrucción.

Runtime safeguards include `paused` mode, per-thread lock, strict length checks, and overflow/underflow-safe counters.

Las salvaguardas en ejecución incluyen modo `paused`, bloqueo por hilo, validación estricta de longitudes y contadores seguros ante overflow/underflow.

## Eligibility Modes

`eligibility_mode` controls who can comment/reply:

`eligibility_mode` controla quién puede comentar/responder:

- `0`: user must be registered and eligible to comment.
- `0`: el usuario debe estar registrado y ser elegible para comentar.

- `1`: user must be registered.
- `1`: el usuario debe estar registrado.

- `2`: any user is allowed.
- `2`: cualquier usuario está permitido.

## Events and Errors

The contract emits lifecycle events for config changes, thread creation/locking, comment create/edit/delete, and reaction add/remove.

El contrato emite eventos de ciclo de vida para cambios de configuración, creación/bloqueo de hilo, creación/edición/borrado de comentarios y alta/baja de reacciones.

Custom errors include authorization failures, paused/locked restrictions, metadata validation failures, eligibility failures, and arithmetic safety checks.

Los errores personalizados incluyen fallos de autorización, restricciones por pausa/bloqueo, fallos de validación de metadatos, fallos de elegibilidad y chequeos de seguridad aritmética.

## Local Development

Build and test with Anchor from this folder:

Compila y prueba con Anchor desde esta carpeta:

```bash
anchor build
anchor test
```

Current `declare_id!` and `Anchor.toml` program addresses are set to local development placeholders and should be replaced before production deployment.

El `declare_id!` actual y las direcciones de programa en `Anchor.toml` están configuradas como placeholders de desarrollo local y deben reemplazarse antes de desplegar a producción.
