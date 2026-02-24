# ATProto Polis Frontend

A Svelte 5 frontend for the ATProto Polis deliberation platform, built with SvelteKit and Tailwind CSS v4.

## Features

- 🔐 **Login** - Authenticate with your Bluesky account
- 📊 **Create Polls** - Start new deliberation topics
- 🔗 **Invite Links** - Share polls with others
- 💬 **Statements** - Add your perspective to polls
- 🗳️ **Voting** - Vote on statements (agree, disagree, pass)
- 🎨 **Modern UI** - Built with Tailwind CSS v4 and custom components
- ⚡ **Svelte 5 Runes** - Using the latest Svelte reactivity system

## Tech Stack

- **SvelteKit** - Full-stack framework
- **Svelte 5** - With runes for reactivity
- **Tailwind CSS v4** - Utility-first CSS framework
- **TypeScript** - Type safety
- **Vite** - Build tool and dev server

## Getting Started

### Prerequisites

- Node.js 22+
- pnpm (or use nix-shell)

### Installation

```bash
# Using nix-shell (recommended on NixOS)
nix-shell -p nodejs_22 pnpm --run "pnpm install"

# Or with pnpm directly
pnpm install
```

### Development

```bash
# Start the dev server
nix-shell -p nodejs_22 pnpm --run "pnpm dev"

# Or
pnpm dev
```

The frontend will be available at `http://localhost:5173`

The backend API is proxied from `http://localhost:3000` to `/api` in development.

### Building for Production

```bash
nix-shell -p nodejs_22 pnpm --run "pnpm build"

# Preview the build
nix-shell -p nodejs_22 pnpm --run "pnpm preview"
```

## Usage

### 1. Login

1. Navigate to the login page
2. Enter your Bluesky handle (e.g., `alice.bsky.social`)
3. Enter your app password (create one in Bluesky settings)
4. Click "Login"

### 2. Create a Poll

1. After logging in, click "Create Poll"
2. Enter a topic (required)
3. Optionally add a description
4. Click "Create Poll"
5. You'll be redirected to the poll page with an invite link

### 3. Share Invite Link

1. On the poll page, copy the invite link
2. Share it with others
3. They can join by pasting the link on the home page

### 4. Add Statements

1. On a poll page, use the "Add a Statement" form
2. Enter your perspective
3. Click "Add Statement"

### 5. Vote on Statements

1. Browse statements on the poll page
2. Click one of the vote buttons:
   - 👍 **Agree** - You agree with the statement
   - 👎 **Disagree** - You disagree with the statement
   - 🤷 **Pass** - You want to skip this statement

## Development Notes

### Svelte 5 Runes

This project uses Svelte 5's new runes syntax:

- `$state()` - Reactive state
- `$derived()` - Computed values
- `$props()` - Component props
- `$bindable()` - Two-way binding

### Tailwind CSS v4

Using the latest Tailwind CSS v4 with `@theme` directive for design tokens.

### API Proxy

The Vite dev server proxies `/api/*` requests to `http://localhost:3000`.

## Troubleshooting

### "Cannot connect to server"

Make sure the backend is running:

```bash
cargo run
```

### Build errors

Ensure Node.js 22+ is installed:

```bash
node --version  # Should be 22.x or higher
```
