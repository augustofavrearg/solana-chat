# WayLearn - Solana LATAM Builders Program

A Solana Anchor program that adds decentralized discussion threads, comments, replies, and reactions to external posts.

Un programa Solana Anchor que agrega hilos de discusión descentralizados, comentarios, respuestas y reacciones a publicaciones externas.

## Prelude

This smart contract was developed as my final project for the WayLearn Solana LATAM Builders Program. Initially, the idea was to create a smart contract that would allow other program participants to upload their projects to the blockchain, using it as storage space. Both the smart contract and the frontend worked correctly, but I later realized that one of the keys to the success of these programs lies in the community we can build among all the participants. Therefore, I felt that a DApp that only displayed information was somewhat lacking in purpose. So, based on my experience as a Web3 developer for a Liquid Staking DApp, I built this smart contract in parallel to provide a comment system where users can express their ideas and exchange opinions about projects with other users and builders.

To create this smart contract, the use of the Codex extension in my Visual Studio Code was crucial, but the following skills were also essential: brainstorming, explaining-code, rust-best-practices, solana-dev, and solana-vulnerability-scanner. They were key from the construction stage, but they were even more important in enriching the development from other perspectives and allowing me to strengthen many concepts about Rust, Anchor, and the Solana blockchain.

---

Este smart contract fue desarrollado como mi proyecto final de WayLearn Solana LATAM Builders Program. En un comienzo la idea original fue armar un smart contract que permitiera a los demas participantes del programa subir sus proyectos a la blockchain usando esta como espacio de almacenamiento, tanto el smart contract como el frontend funcionaron correctamente, pero luego crei que una de las claves del exito de estos programas esta en la comunidad que podemos generar entre todos los participantes, por ello, crei que una DApp que solamente mostrase informacion estaba un poco vacia de proposito y fue asi que, basado en mi experiencia como Web3 developer de una Liquid Staking DApp, construi este smart contract en paralelo para aportar un sistema de comentarios en donde los usuarios puedan expresar sus ideas e intercambiar opiniones acerca de los proyectos con otros usuarios y builders.

Para generar este smart contract fue clave el uso de la extension de Codex en mi Visual Studio Code pero tambien fueron clave el uso de los siguientes skills: brainstorming, explaining-code, rust-best-practices, solana-dev y sonala-vulnerability-scanner. Fueron clave desde la construccion pero fueron aun mas importantes para enriquecer el desarrollo desde otras perspectivas y permitirme fortalecer muchos conceptos acerca de Rust, Anchor y la blockchain de Solana.

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

## Frontend Integration Guide

Use Anchor TS in your frontend to connect a wallet, instantiate `Program`, derive PDAs, and call instructions.

Minimal setup example:
```ts
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
// import { ProjectComments } from "./idl/project_comments"; // generated types

const provider = new anchor.AnchorProvider(connection, wallet, {});
const program = new anchor.Program(idl, provider); // as Program<ProjectComments>
```

### 1) Create or derive a thread for your post

Derive the thread PDA with seeds: `["thread", sourceProgramId, postAccount]`.

```ts
const [threadPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("thread"), sourceProgramId.toBuffer(), postAccount.toBuffer()],
  program.programId
);
```

Call `createThreadForPost(sourceProgramId, postId)` once per post:
```ts
await program.methods
  .createThreadForPost(sourceProgramId, new anchor.BN(postId))
  .accounts({
    config: configPda,
    thread: threadPda,
    postMeta,
    postAccount,
    authority: wallet.publicKey,
    instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### 2) Create comments and replies

Top-level comment:
```ts
await program.methods
  .createComment(sourceProgramId, body)
  .accounts({
    config: configPda,
    thread: threadPda,
    comment: commentPda,
    userMeta,
    author: wallet.publicKey,
    instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

Reply to an existing comment:
```ts
await program.methods
  .replyComment(sourceProgramId, body)
  .accounts({
    config: configPda,
    thread: threadPda,
    parentComment,
    comment: replyPda,
    userMeta,
    author: wallet.publicKey,
    instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### 3) Query comments for a thread

Fetch all comments and filter by `thread` field:
```ts
const comments = await program.account.comment.all([
  {
    memcmp: {
      // discriminator(8) + thread starts at offset 8
      offset: 8,
      bytes: threadPda.toBase58(),
    },
  },
]);
```

Sort by `commentId` client-side to render deterministic order.

### 4) Add/remove reactions and query them

Reaction PDA seeds: `["reaction", comment, user, reactionKind]`.

```ts
await program.methods
  .addReaction("like", expectedCpiProgram)
  .accounts({
    config: configPda,
    thread: threadPda,
    comment: commentPda,
    reaction: reactionPda,
    user: wallet.publicKey,
    pinnedCpiProgram: expectedCpiProgram,
    instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

Query reactions for a comment:
```ts
const reactions = await program.account.reaction.all([
  {
    memcmp: {
      // discriminator(8) + comment starts at offset 8
      offset: 8,
      bytes: commentPda.toBase58(),
    },
  },
]);
```

---

Usa Anchor TS en tu frontend para conectar wallet, instanciar `Program`, derivar PDAs y ejecutar instrucciones.

Ejemplo mínimo de setup:
```ts
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
// import { ProjectComments } from "./idl/project_comments"; // tipos generados

const provider = new anchor.AnchorProvider(connection, wallet, {});
const program = new anchor.Program(idl, provider); // como Program<ProjectComments>
```

### 1) Crear o derivar un thread para tu post

Deriva la PDA del thread con seeds: `["thread", sourceProgramId, postAccount]`.

```ts
const [threadPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("thread"), sourceProgramId.toBuffer(), postAccount.toBuffer()],
  program.programId
);
```

Llamá `createThreadForPost(sourceProgramId, postId)` una vez por post:
```ts
await program.methods
  .createThreadForPost(sourceProgramId, new anchor.BN(postId))
  .accounts({
    config: configPda,
    thread: threadPda,
    postMeta,
    postAccount,
    authority: wallet.publicKey,
    instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### 2) Crear comentarios y respuestas

Comentario de primer nivel:
```ts
await program.methods
  .createComment(sourceProgramId, body)
  .accounts({
    config: configPda,
    thread: threadPda,
    comment: commentPda,
    userMeta,
    author: wallet.publicKey,
    instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

Respuesta a un comentario existente:
```ts
await program.methods
  .replyComment(sourceProgramId, body)
  .accounts({
    config: configPda,
    thread: threadPda,
    parentComment,
    comment: replyPda,
    userMeta,
    author: wallet.publicKey,
    instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### 3) Consultar comentarios de un thread

Traé todos los comentarios y filtrá por el campo `thread`:
```ts
const comments = await program.account.comment.all([
  {
    memcmp: {
      // discriminator(8) + thread arranca en offset 8
      offset: 8,
      bytes: threadPda.toBase58(),
    },
  },
]);
```

Ordená por `commentId` del lado cliente para render estable.

### 4) Agregar/quitar reacciones y consultarlas

Seeds de PDA de reacción: `["reaction", comment, user, reactionKind]`.

```ts
await program.methods
  .addReaction("like", expectedCpiProgram)
  .accounts({
    config: configPda,
    thread: threadPda,
    comment: commentPda,
    reaction: reactionPda,
    user: wallet.publicKey,
    pinnedCpiProgram: expectedCpiProgram,
    instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

Consultar reacciones de un comentario:
```ts
const reactions = await program.account.reaction.all([
  {
    memcmp: {
      // discriminator(8) + comment arranca en offset 8
      offset: 8,
      bytes: commentPda.toBase58(),
    },
  },
]);
```

## Local Development

Build and test with Anchor from this folder:
```bash
anchor build
anchor test
```

Current `declare_id!` and `Anchor.toml` program addresses are set to local development placeholders and should be replaced before production deployment.

Compila y prueba con Anchor desde esta carpeta:
El `declare_id!` actual y las direcciones de programa en `Anchor.toml` están configuradas como placeholders de desarrollo local y deben reemplazarse antes de desplegar a producción.
