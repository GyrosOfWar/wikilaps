<script lang="ts">
  import { type MessageLike } from "@inlang/paraglide-js-svelte";

  interface Props {
    message: MessageLike;
    inputs: Record<string, never>;
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  let { message, inputs }: Props = $props();
</script>

<!-- <ParaglideMessage {message} {inputs}>
  {#snippet link({ children, options })}
    <a
      class="text-primary-500 underline decoration-primary-500/40 underline-offset-2 hover:decoration-primary-500"
      target={options.external ? "_blank" : undefined}
      href={options.to as string}
    >
      {@render children?.()}
    </a>
  {/snippet}
</ParaglideMessage> -->
