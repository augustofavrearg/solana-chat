# WayLearn - Solana LATAM Builders Program

A Solana Anchor program that adds decentralized discussion threads, comments, replies, and reactions to external posts.

Un programa Solana Anchor que agrega hilos de discusión descentralizados, comentarios, respuestas y reacciones a publicaciones externas.

## Features

- Creates one discussion thread PDA per external post.
- Supports top-level comments and nested replies.
- Supports per-user reactions on comments using deterministic PDAs.
- Includes moderation controls: comment deletion, thread locking, and admin overrides.
- Supports runtime governance via config updates (`paused`, eligibility mode, limits).

---

- Crea una PDA de hilo de discusión por cada publicación externa.
- Soporta comentarios de primer nivel y respuestas anidadas.
- Soporta reacciones por usuario en comentarios usando PDAs determinísticas.
- Incluye controles de moderación: borrado de comentarios, bloqueo de hilos y acciones administrativas.
- Soporta gobernanza en tiempo de ejecución mediante actualización de configuración (`paused`, modo de elegibilidad, límites).

## Program Architecture

The program stores global settings in `Config`, thread metadata in `Thread`, message content in `Comment`, and user reactions in `Reaction`.
Main PDA seeds are: `config`, `thread + source_program_id + post_account`, `comment + thread + comment_id`, and `reaction + comment + user + reaction_kind`.

El programa guarda la configuración global en `Config`, metadatos del hilo en `Thread`, contenido de mensajes en `Comment` y reacciones de usuario en `Reaction`.
Las seeds principales de PDA son: `config`, `thread + source_program_id + post_account`, `comment + thread + comment_id` y `reaction + comment + user + reaction_kind`.

## Instructions

- `initialize_config`: initializes global configuration and sets the super admin.
- `update_config`: updates pause state, eligibility mode, and limits.
- `create_thread_for_post`: creates a thread linked to an external post after validating `PostMetaStandard`.
- `create_comment`: creates a top-level comment after validating user eligibility.
- `reply_comment`: creates a reply linked to a parent comment and root comment.
- `edit_comment`: lets only the original author update the comment body.
- `delete_comment`: allows deletion by comment author, post author, or super admin (soft delete to `[deleted]`).
- `set_thread_lock`: allows post author or super admin to lock/unlock a thread.
- `add_reaction`: creates a reaction account and increments comment reaction counter.
- `remove_reaction`: removes caller-owned reaction and decrements counter.
- `admin_remove_reaction`: allows post author or super admin to remove any reaction.

---

- `initialize_config`: inicializa la configuración global y define el super admin.
- `update_config`: actualiza estado de pausa, modo de elegibilidad y límites.
- `create_thread_for_post`: crea un hilo vinculado a una publicación externa tras validar `PostMetaStandard`.
- `create_comment`: crea un comentario de primer nivel tras validar elegibilidad del usuario.
- `reply_comment`: crea una respuesta vinculada al comentario padre y al comentario raíz.
- `edit_comment`: permite solo al autor original actualizar el cuerpo del comentario.
- `delete_comment`: permite borrar por autor del comentario, autor del post o super admin (borrado lógico a `[deleted]`).
- `set_thread_lock`: permite al autor del post o super admin bloquear/desbloquear un hilo.
- `add_reaction`: crea una cuenta de reacción e incrementa el contador de reacciones del comentario.
- `remove_reaction`: elimina la reacción del usuario que llama y decrementa el contador.
- `admin_remove_reaction`: permite al autor del post o super admin eliminar cualquier reacción.

## Security Model

Every write path validates signer presence and rejects unauthorized actors for privileged operations.
The program validates account owners for external metadata (`post_meta`, `user_meta`) and enforces canonical PDA derivation.
The Instructions sysvar is validated and introspected to harden instruction context checks.
Runtime safeguards include `paused` mode, per-thread lock, strict length checks, and overflow/underflow-safe counters.

Cada ruta de escritura valida la firma requerida y rechaza actores no autorizados en operaciones privilegiadas.
El programa valida propietarios de cuentas para metadatos externos (`post_meta`, `user_meta`) y exige derivación canónica de PDAs.
Se valida e inspecciona el sysvar de Instructions para reforzar los chequeos del contexto de instrucción.
Las salvaguardas en ejecución incluyen modo `paused`, bloqueo por hilo, validación estricta de longitudes y contadores seguros ante overflow/underflow.

## Eligibility Modes

`eligibility_mode` controls who can comment/reply:
- `0`: user must be registered and eligible to comment.
- `1`: user must be registered.
- `2`: any user is allowed.

---

`eligibility_mode` controla quién puede comentar/responder:
- `0`: el usuario debe estar registrado y ser elegible para comentar.
- `1`: el usuario debe estar registrado.
- `2`: cualquier usuario está permitido.

## Events and Errors

The contract emits lifecycle events for config changes, thread creation/locking, comment create/edit/delete, and reaction add/remove.
Custom errors include authorization failures, paused/locked restrictions, metadata validation failures, eligibility failures, and arithmetic safety checks.

El contrato emite eventos de ciclo de vida para cambios de configuración, creación/bloqueo de hilo, creación/edición/borrado de comentarios y alta/baja de reacciones.
Los errores personalizados incluyen fallos de autorización, restricciones por pausa/bloqueo, fallos de validación de metadatos, fallos de elegibilidad y chequeos de seguridad aritmética.

## Local Development

Build and test with Anchor from this folder:
```bash
anchor build
anchor test
```

Current `declare_id!` and `Anchor.toml` program addresses are set to local development placeholders and should be replaced before production deployment.

Compila y prueba con Anchor desde esta carpeta:
El `declare_id!` actual y las direcciones de programa en `Anchor.toml` están configuradas como placeholders de desarrollo local y deben reemplazarse antes de desplegar a producción.
