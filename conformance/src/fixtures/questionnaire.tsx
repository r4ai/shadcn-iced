import {
  Questionnaire,
  QuestionnaireActions,
  QuestionnaireChoice,
  QuestionnaireChoiceDescription,
  QuestionnaireChoices,
  QuestionnaireDescription,
  QuestionnaireError,
  QuestionnaireInput,
  QuestionnaireItem,
  QuestionnaireNext,
  QuestionnairePrevious,
  QuestionnaireProgress,
  QuestionnaireSkip,
  QuestionnaireSubmit,
  QuestionnaireTitle,
} from "@/components/ui/questionnaire"
import type { Fixtures } from "@/fixtures"

const items = [
  {
    name: "direction",
    required: true,
    choices: [
      { value: "Tool call timeline" },
      { value: "Approval checkpoints" },
      { value: "Sub-agent handoffs" },
    ],
  },
  {
    name: "signals",
    choices: [
      { value: "Progress" },
      { value: "Decisions" },
      { value: "Risks" },
      { value: "Next step" },
    ],
  },
  {
    name: "timing",
    required: true,
    choices: [
      { value: "Start now" },
      { value: "Next development cycle" },
      { value: "Add it to the backlog" },
    ],
  },
]

const descriptions: Record<string, string> = {
  "Tool call timeline": "Show what the agent ran and what came back.",
  "Approval checkpoints": "Ask before sensitive or destructive actions.",
  "Sub-agent handoffs": "Make delegated work and results easier to follow.",
}

export default {
  flow: () => <Demo />,
  skip: () => <Demo />,
} satisfies Fixtures

function Demo() {
  return (
    <div className="w-full max-w-md">
      <Questionnaire
        items={items}
        shortcuts="letters"
        onSubmit={(event) => event.preventDefault()}
      >
        <QuestionnaireProgress />
        <QuestionnaireItem name="direction" required>
          <QuestionnaireTitle>
            What should the agent build next?
          </QuestionnaireTitle>
          <QuestionnaireDescription>
            Choose a direction or describe another task.
          </QuestionnaireDescription>
          <QuestionnaireChoices>
            {items[0].choices.map((choice) => (
              <QuestionnaireChoice key={choice.value} value={choice.value}>
                <span>{choice.value}</span>
                <QuestionnaireChoiceDescription>
                  {descriptions[choice.value]}
                </QuestionnaireChoiceDescription>
              </QuestionnaireChoice>
            ))}
            <QuestionnaireInput placeholder="Describe another feature…" />
          </QuestionnaireChoices>
          <QuestionnaireError>Choose an answer to continue.</QuestionnaireError>
        </QuestionnaireItem>
        <QuestionnaireItem name="signals" multiple>
          <QuestionnaireTitle>
            What should every progress update include?
          </QuestionnaireTitle>
          <QuestionnaireDescription>
            Select all that apply, or skip this question.
          </QuestionnaireDescription>
          <QuestionnaireChoices>
            {items[1].choices.map((choice) => (
              <QuestionnaireChoice key={choice.value} value={choice.value}>
                <span>{choice.value}</span>
              </QuestionnaireChoice>
            ))}
          </QuestionnaireChoices>
          <QuestionnaireError>
            Choose an answer or skip this question.
          </QuestionnaireError>
        </QuestionnaireItem>
        <QuestionnaireItem name="timing" required>
          <QuestionnaireTitle>When should work begin?</QuestionnaireTitle>
          <QuestionnaireDescription>
            Choose when the agent should begin the work.
          </QuestionnaireDescription>
          <QuestionnaireChoices>
            {items[2].choices.map((choice) => (
              <QuestionnaireChoice key={choice.value} value={choice.value}>
                <span>{choice.value}</span>
              </QuestionnaireChoice>
            ))}
          </QuestionnaireChoices>
          <QuestionnaireError>Choose an answer to continue.</QuestionnaireError>
        </QuestionnaireItem>
        <QuestionnaireActions>
          <QuestionnairePrevious />
          <QuestionnaireSkip />
          <QuestionnaireNext />
          <QuestionnaireSubmit />
        </QuestionnaireActions>
      </Questionnaire>
    </div>
  )
}
