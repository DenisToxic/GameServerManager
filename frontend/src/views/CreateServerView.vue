<script setup lang="ts">
import { computed, ref } from 'vue';
import { useServerStore } from '../stores/serverStore';
import { useRouter } from 'vue-router';
import { ArrowLeft } from 'lucide-vue-next';

type GameKind = 'minecraft' | 'rust' | 'hytale';

interface GamePreset {
  kind: GameKind;
  label: string;
  description: string;
  defaultPort: number;
  defaultMemoryMb: number;
  defaultCpuPercent: number;
}

const GAME_PRESETS: GamePreset[] = [
  {
    kind: 'minecraft',
    label: 'Minecraft',
    description: 'Create a Didstopia Minecraft server with optional RCON and startup overrides.',
    defaultPort: 25565,
    defaultMemoryMb: 4096,
    defaultCpuPercent: 200,
  },
  {
    kind: 'rust',
    label: 'Rust',
    description: 'Configure first-boot Rust server settings like player count, map size, RCON, and startup args.',
    defaultPort: 28015,
    defaultMemoryMb: 6144,
    defaultCpuPercent: 300,
  },
  {
    kind: 'hytale',
    label: 'Hytale',
    description: 'Set the boot command and baseline server limits for the current Hytale container preset.',
    defaultPort: 25565,
    defaultMemoryMb: 4096,
    defaultCpuPercent: 200,
  },
];

const store = useServerStore();
const router = useRouter();

const form = ref({
  name: '',
  nodeId: 1,
  gameKind: 'minecraft' as GameKind,
  allocatedPort: 25565,
  memoryLimitMb: 4096,
  cpuLimitPercent: 200,
  minecraft: {
    startupArguments: 'nogui',
    rconEnabled: false,
    rconPort: 25575,
    rconPassword: '',
    customJar: '',
  },
  rust: {
    description: '',
    maxPlayers: 50,
    worldSize: 3500,
    seed: 12345,
    rconPassword: '',
    rconPort: 28016,
    queryPort: 28016,
    appPort: 28082,
    websiteUrl: '',
    startupArguments: '',
    modFramework: 'vanilla',
  },
  hytale: {
    maxPlayers: 32,
    startupCommand: 'sleep infinity',
  },
});

const isSubmitting = ref(false);
const submitError = ref<string | null>(null);

const selectedPreset = computed(
  () => GAME_PRESETS.find((preset) => preset.kind === form.value.gameKind) ?? GAME_PRESETS[0],
);

function applyPreset(kind: GameKind) {
  const preset = GAME_PRESETS.find((entry) => entry.kind === kind);
  if (!preset) return;

  form.value.gameKind = preset.kind;
  form.value.allocatedPort = preset.defaultPort;
  form.value.memoryLimitMb = preset.defaultMemoryMb;
  form.value.cpuLimitPercent = preset.defaultCpuPercent;
}

function buildServerSettings() {
  if (form.value.gameKind === 'minecraft') {
    return {
      startup_arguments: form.value.minecraft.startupArguments,
      rcon_enabled: form.value.minecraft.rconEnabled,
      rcon_port: form.value.minecraft.rconPort,
      rcon_password: form.value.minecraft.rconPassword,
      custom_jar: form.value.minecraft.customJar,
    };
  }

  if (form.value.gameKind === 'rust') {
    return {
      description: form.value.rust.description,
      max_players: form.value.rust.maxPlayers,
      world_size: form.value.rust.worldSize,
      seed: form.value.rust.seed,
      rcon_password: form.value.rust.rconPassword,
      rcon_port: form.value.rust.rconPort,
      query_port: form.value.rust.queryPort,
      app_port: form.value.rust.appPort,
      website_url: form.value.rust.websiteUrl,
      startup_arguments: form.value.rust.startupArguments,
      mod_framework: form.value.rust.modFramework,
    };
  }

  return {
    max_players: form.value.hytale.maxPlayers,
    startup_command: form.value.hytale.startupCommand,
  };
}

const handleSubmit = async () => {
  isSubmitting.value = true;
  submitError.value = null;

  try {
    await store.createServer({
      name: form.value.name,
      node_id: form.value.nodeId,
      game_kind: form.value.gameKind,
      server_settings: buildServerSettings(),
      allocated_port: form.value.allocatedPort,
      memory_limit_mb: form.value.memoryLimitMb,
      cpu_limit_percent: form.value.cpuLimitPercent,
    });
    router.push('/');
  } catch (err: any) {
    submitError.value = err.response?.data?.error || store.error || 'Failed to create server';
  } finally {
    isSubmitting.value = false;
  }
};
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <div class="mb-6 flex items-center gap-4">
      <button @click="router.push('/')" class="p-2 hover:bg-surface rounded-md transition-colors text-gray-400 hover:text-white">
        <ArrowLeft class="w-5 h-5" />
      </button>
      <div>
        <h1 class="text-3xl font-bold text-white">Create Server</h1>
        <p class="text-sm text-gray-400">Pick a game first, then tune the options that actually matter for that server type.</p>
      </div>
    </div>

    <div class="space-y-6">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <button
          v-for="preset in GAME_PRESETS"
          :key="preset.kind"
          type="button"
          @click="applyPreset(preset.kind)"
          class="rounded-xl border p-4 text-left transition-colors"
          :class="form.gameKind === preset.kind ? 'border-primary bg-primary/10' : 'border-surface-hover bg-surface hover:border-primary/50'"
        >
          <div class="text-lg font-semibold text-white">{{ preset.label }}</div>
          <p class="mt-2 text-sm text-gray-400">{{ preset.description }}</p>
          <div class="mt-4 text-xs text-gray-500">
            Port {{ preset.defaultPort }} | {{ preset.defaultMemoryMb }} MB | {{ preset.defaultCpuPercent }}% CPU
          </div>
        </button>
      </div>

      <div class="bg-surface rounded-xl border border-surface-hover p-6 md:p-8">
        <form @submit.prevent="handleSubmit" class="space-y-6">
          <div v-if="submitError" class="rounded-md border border-danger/30 bg-danger/10 p-3 text-sm text-danger">
            {{ submitError }}
          </div>

          <div class="rounded-md border border-primary/20 bg-primary/10 p-3 text-sm text-gray-300">
            The first creation/start for a game can take a while because the image and game files may need to download first.
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div class="space-y-2 md:col-span-2">
              <label class="text-sm font-medium text-gray-300">Server Name</label>
              <input
                v-model="form.name"
                type="text"
                class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
                :placeholder="`${selectedPreset.label} server name`"
                required
              />
            </div>

            <div class="space-y-2">
              <label class="text-sm font-medium text-gray-300">Allocated Port</label>
              <input
                v-model.number="form.allocatedPort"
                type="number"
                class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
                min="1"
                max="65535"
                required
              />
            </div>

            <div class="space-y-2">
              <label class="text-sm font-medium text-gray-300">Memory Limit (MB)</label>
              <input
                v-model.number="form.memoryLimitMb"
                type="number"
                class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
                min="128"
                required
              />
            </div>

            <div class="space-y-2">
              <label class="text-sm font-medium text-gray-300">CPU Limit (%)</label>
              <input
                v-model.number="form.cpuLimitPercent"
                type="number"
                class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
                min="1"
                max="1000"
                required
              />
            </div>
          </div>

          <div v-if="form.gameKind === 'minecraft'" class="rounded-xl border border-surface-hover bg-background/50 p-5">
            <h2 class="text-lg font-semibold text-white">Minecraft Settings</h2>
            <div class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-2 md:col-span-2">
                <label class="text-sm font-medium text-gray-300">Startup Arguments</label>
                <input
                  v-model="form.minecraft.startupArguments"
                  type="text"
                  class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200"
                />
              </div>
              <div class="space-y-2 md:col-span-2">
                <label class="text-sm font-medium text-gray-300">Custom Jar Path</label>
                <input
                  v-model="form.minecraft.customJar"
                  type="text"
                  placeholder="/app/custom.jar"
                  class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200"
                />
              </div>
              <label class="flex items-center gap-3 text-sm text-gray-300">
                <input v-model="form.minecraft.rconEnabled" type="checkbox" class="h-4 w-4 rounded border-surface-hover bg-background" />
                Enable RCON
              </label>
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">RCON Port</label>
                <input
                  v-model.number="form.minecraft.rconPort"
                  type="number"
                  class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200"
                  min="1"
                  max="65535"
                />
              </div>
              <div class="space-y-2 md:col-span-2">
                <label class="text-sm font-medium text-gray-300">RCON Password</label>
                <input
                  v-model="form.minecraft.rconPassword"
                  type="text"
                  class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200"
                />
              </div>
            </div>
          </div>

          <div v-else-if="form.gameKind === 'rust'" class="rounded-xl border border-surface-hover bg-background/50 p-5">
            <h2 class="text-lg font-semibold text-white">Rust Settings</h2>
            <div class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-2 md:col-span-2">
                <label class="text-sm font-medium text-gray-300">Description</label>
                <input
                  v-model="form.rust.description"
                  type="text"
                  class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200"
                />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">Max Players</label>
                <input v-model.number="form.rust.maxPlayers" type="number" min="1" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">World Size</label>
                <input v-model.number="form.rust.worldSize" type="number" min="1000" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">World Seed</label>
                <input v-model.number="form.rust.seed" type="number" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">Website URL</label>
                <input v-model="form.rust.websiteUrl" type="url" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
              <div class="space-y-2 md:col-span-2">
                <label class="text-sm font-medium text-gray-300">Startup Arguments</label>
                <input v-model="form.rust.startupArguments" type="text" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">RCON Port</label>
                <input v-model.number="form.rust.rconPort" type="number" min="1" max="65535" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">Query Port</label>
                <input v-model.number="form.rust.queryPort" type="number" min="1" max="65535" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">Rust+ App Port</label>
                <input v-model.number="form.rust.appPort" type="number" min="1" max="65535" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">Mod Framework</label>
                <select v-model="form.rust.modFramework" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200">
                  <option value="vanilla">Vanilla</option>
                  <option value="oxide">Oxide</option>
                  <option value="carbon">Carbon</option>
                </select>
              </div>
              <div class="space-y-2 md:col-span-2">
                <label class="text-sm font-medium text-gray-300">RCON Password</label>
                <input v-model="form.rust.rconPassword" type="text" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
            </div>
          </div>

          <div v-else class="rounded-xl border border-surface-hover bg-background/50 p-5">
            <h2 class="text-lg font-semibold text-white">Hytale Settings</h2>
            <div class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-2">
                <label class="text-sm font-medium text-gray-300">Max Players</label>
                <input v-model.number="form.hytale.maxPlayers" type="number" min="1" class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200" />
              </div>
              <div class="space-y-2 md:col-span-2">
                <label class="text-sm font-medium text-gray-300">Startup Command</label>
                <input
                  v-model="form.hytale.startupCommand"
                  type="text"
                  class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200"
                />
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-surface-hover bg-background/50 p-4 text-sm text-gray-400">
            {{ selectedPreset.description }}
          </div>

          <div class="pt-4 flex justify-end gap-3">
            <button
              type="button"
              @click="router.push('/')"
              class="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white hover:bg-surface-hover rounded-md transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              :disabled="isSubmitting"
              class="bg-primary hover:bg-primary-hover text-white font-medium py-2 px-6 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center min-w-[160px]"
            >
              <span v-if="isSubmitting" class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
              <span v-if="isSubmitting">Provisioning {{ selectedPreset.label }}...</span>
              <span v-else>Create {{ selectedPreset.label }}</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>
