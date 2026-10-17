import { Bubble, BubbleContent } from "@/components/ui/bubble"
import { Message, MessageContent } from "@/components/ui/message"
import {
  MessageScroller,
  MessageScrollerButton,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerProvider,
  MessageScrollerViewport,
} from "@/components/ui/message-scroller"
import type { Fixtures } from "@/fixtures"

const messages = [
  {
    id: "1",
    role: "user",
    content: "How do I keep a chat pinned to the newest message?",
  },
  {
    id: "2",
    role: "assistant",
    content:
      "Wrap the message list in a message scroller. The viewport stays at the bottom as new text arrives, so readers always see the latest reply land in place.",
  },
  {
    id: "3",
    role: "user",
    content: "And if the reader scrolled up to re-read an older answer?",
  },
  {
    id: "4",
    role: "assistant",
    content:
      "Then it does not yank them back down. A scroll button appears at the bottom of the viewport, and one press returns to the newest message.",
  },
  { id: "5", role: "user", content: "Does it work while streaming?" },
  {
    id: "6",
    role: "assistant",
    content:
      "Yes. It only follows the stream while the reader is already at the bottom of the conversation.",
  },
  { id: "7", role: "user", content: "That works, thanks!" },
]

function Demo() {
  return (
    <div className="h-80 w-full max-w-sm">
      <MessageScrollerProvider>
        <MessageScroller>
          <MessageScrollerViewport>
            <MessageScrollerContent className="p-4">
              {messages.map((message) => (
                <MessageScrollerItem
                  key={message.id}
                  messageId={message.id}
                  scrollAnchor={message.role === "user"}
                >
                  {message.role === "user" ? (
                    <Message align="end">
                      <MessageContent>
                        <Bubble>
                          <BubbleContent>{message.content}</BubbleContent>
                        </Bubble>
                      </MessageContent>
                    </Message>
                  ) : (
                    <Message>
                      <MessageContent>
                        <Bubble variant="ghost">
                          <BubbleContent>{message.content}</BubbleContent>
                        </Bubble>
                      </MessageContent>
                    </Message>
                  )}
                </MessageScrollerItem>
              ))}
            </MessageScrollerContent>
          </MessageScrollerViewport>
          <MessageScrollerButton />
        </MessageScroller>
      </MessageScrollerProvider>
    </div>
  )
}

export default {
  bottom: () => <Demo />,
  "scroll-to-start": () => <Demo />,
} satisfies Fixtures
