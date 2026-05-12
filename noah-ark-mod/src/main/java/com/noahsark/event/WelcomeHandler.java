package com.noahsark.event;

import net.minecraft.component.DataComponentTypes;
import net.minecraft.component.type.WrittenBookContentComponent;
import net.minecraft.item.ItemStack;
import net.minecraft.item.Items;
import net.minecraft.server.network.ServerPlayerEntity;
import net.minecraft.text.RawFilteredPair;
import net.minecraft.text.Text;

import java.util.List;

public final class WelcomeHandler {
    private static final String BOOK_TITLE  = "NULL";
    private static final String BOOK_AUTHOR = "Fragment";

    private WelcomeHandler() {}

    public static void onJoin(ServerPlayerEntity player) {
        if (hasBook(player)) return;

        // Welcome message
        player.sendMessage(Text.literal(
            "§8[§5Noah's Ark§8] §7The storm is already coming. It found you. It always does."), false);

        // Give the guide book
        ItemStack book = new ItemStack(Items.WRITTEN_BOOK);
        book.set(DataComponentTypes.WRITTEN_BOOK_CONTENT, new WrittenBookContentComponent(
            RawFilteredPair.of(BOOK_TITLE),
            BOOK_AUTHOR,
            0,
            buildPages(),
            true
        ));
        player.getInventory().offerOrDrop(book);
    }

    private static boolean hasBook(ServerPlayerEntity player) {
        for (ItemStack stack : player.getInventory().getMainStacks()) {
            if (stack.isOf(Items.WRITTEN_BOOK)) {
                WrittenBookContentComponent content = stack.get(DataComponentTypes.WRITTEN_BOOK_CONTENT);
                if (content != null && BOOK_TITLE.equals(content.title().raw())
                        && BOOK_AUTHOR.equals(content.author())) {
                    return true;
                }
            }
        }
        return false;
    }

    private static List<RawFilteredPair<Text>> buildPages() {
        return List.of(
            page(
                "§lNULL\n§r§8by Fragment\n\n" +
                "If you're reading this, you survived long enough to find it.\n\n" +
                "I didn't.\n\n" +
                "But I left everything I knew behind, so that maybe you could."
            ),
            page(
                "§lThe Storm\n\n" +
                "§rIt comes without warning. Rain first — then the slurp juice falls.\n\n" +
                "You have one day before the flood rises. Maybe less if you're slow.\n\n" +
                "Do not waste it."
            ),
            page(
                "§lGopher Wood\n\n" +
                "§rThose crooked trees that grow anywhere the soil shows — that's gopher wood.\n\n" +
                "Can't be planked. Doesn't care. It's what the ark is made of.\n\n" +
                "Find them. Cut them. Keep them."
            ),
            page(
                "§lThe Slurp Juice\n\n" +
                "§rIt rains down in blobs before the flood.\n\n" +
                "Collect it in a bucket. It will give you the §5Slurpy§r effect if you stay in it — " +
                "your body starts to change.\n\n" +
                "I lost two people to that."
            ),
            page(
                "§lSlurp Shield Plate\n§8[Recipe]\n\n" +
                "§rIron · Slurp Bucket · Iron\n\n" +
                "Yields 3 plates. Lines the hull of the ark.\n\n" +
                "Also the thing keeping the juice off you when you swim in it."
            ),
            page(
                "§lSlurp Shield\n§8[Recipe]\n\n" +
                "§r · P ·\n" +
                "P G P\n" +
                "G G G\n\n" +
                "§8P = Shield Plate or Slurp Bucket\n" +
                "G = Gopher Wood\n\n" +
                "§rHold it. The Slurpy effect won't touch you."
            ),
            page(
                "§lNoah's Ark\n§8[Recipe]\n\n" +
                "§rI · W · I\n" +
                "S · B · S\n" +
                "G G G\n\n" +
                "§8I=Iron  W=Wool  S=Slurp Bucket\n" +
                "B=Any Boat  G=Gopher Wood\n\n" +
                "§rPlace it. It will build itself."
            ),
            page(
                "§lThe §5Slurpy §rEffect\n\n" +
                "Stay submerged in slurp juice for 1.5 seconds.\n\n" +
                "Your skin changes. Your body forgets what it was.\n\n" +
                "The shield stops it. Nothing else does.\n\n" +
                "I know. I tried everything else."
            ),
            page(
                "§lWhat I Wish I'd Known\n\n" +
                "§r- Build the ark §lbefore§r the flood.\n" +
                "- Collect slurp blobs §learly§r.\n" +
                "- Stay on high ground once the water rises.\n" +
                "- §lDo not§r touch the juice without a shield.\n\n" +
                "That's all."
            ),
            page(
                "§rI don't know if you'll make it.\n\n" +
                "I didn't.\n\n" +
                "But you're still here.\n\n" +
                "That's already more than I managed on day one.\n\n\n" +
                "§8— Fragment"
            )
        );
    }

    private static RawFilteredPair<Text> page(String content) {
        return RawFilteredPair.of(Text.literal(content));
    }
}
