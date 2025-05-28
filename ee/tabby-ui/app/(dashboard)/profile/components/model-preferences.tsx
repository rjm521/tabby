'use client'

import React, { useEffect, useState } from 'react'
import { useMutation, useQuery } from 'urql'

import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import { Separator } from '@/components/ui/separator'
import { graphql } from '@/lib/gql'
import { toast } from '@/components/ui/use-toast'

import { ProfileCard } from './profile-card'

// GraphQL Operations
const GetUserModelPreferences = graphql(/* GraphQL */ `
  query GetUserModelPreferences {
    userModelPreferences {
      completionModel
      chatModel
      updatedAt
    }
  }
`)

const GetAvailableModels = graphql(/* GraphQL */ `
  query GetAvailableModels($type: ModelTypeEnum) {
    availableModels(type: $type) {
      modelName
      description
      performanceTier
    }
  }
`)

const UpdateUserModelPreferencesMutation = graphql(/* GraphQL */ `
  mutation UpdateUserModelPreferences($input: UpdateUserModelPreferencesInput!) {
    updateUserModelPreferences(input: $input) {
      completionModel
      chatModel
      updatedAt
    }
  }
`)

const ResetUserModelPreferencesMutation = graphql(/* GraphQL */ `
  mutation ResetUserModelPreferences {
    resetUserModelPreferences {
      completionModel
      chatModel
      updatedAt
    }
  }
`)

interface Model {
  modelName: string
  description?: string | null
  performanceTier?: string | null
}

export function ModelPreferences() {
  const [preferencesResult, reexecuteQueryPreferences] = useQuery({
    query: GetUserModelPreferences
  })
  const [completionModelsResult] = useQuery({
    query: GetAvailableModels,
    variables: { type: 'COMPLETION' }
  })
  const [chatModelsResult] = useQuery({
    query: GetAvailableModels,
    variables: { type: 'CHAT' }
  })

  const [updateResult, executeUpdate] = useMutation(
    UpdateUserModelPreferencesMutation
  )
  const [resetResult, executeReset] = useMutation(
    ResetUserModelPreferencesMutation
  )

  const [selectedCompletionModel, setSelectedCompletionModel] = useState<string>('')
  const [selectedChatModel, setSelectedChatModel] = useState<string>('')

  useEffect(() => {
    if (preferencesResult.data?.userModelPreferences) {
      setSelectedCompletionModel(
        preferencesResult.data.userModelPreferences.completionModel || ''
      )
      setSelectedChatModel(
        preferencesResult.data.userModelPreferences.chatModel || ''
      )
    }
  }, [preferencesResult.data])

  const handleSubmit = async () => {
    const input = {
      completionModel: selectedCompletionModel || null, // Send null if empty string
      chatModel: selectedChatModel || null
    }
    const result = await executeUpdate({ input })
    if (result.error) {
      toast({
        title: 'Error',
        description: 'Failed to update model preferences.',
        variant: 'destructive'
      })
    } else {
      toast({
        title: 'Success',
        description: 'Model preferences updated.'
      })
      reexecuteQueryPreferences({ requestPolicy: 'network-only' })
    }
  }

  const handleReset = async () => {
    const result = await executeReset({})
    if (result.error) {
      toast({
        title: 'Error',
        description: 'Failed to reset model preferences.',
        variant: 'destructive'
      })
    } else {
      toast({
        title: 'Success',
        description: 'Model preferences reset to default.'
      })
      setSelectedCompletionModel('')
      setSelectedChatModel('')
      reexecuteQueryPreferences({ requestPolicy: 'network-only' })
    }
  }

  const isLoading =
    preferencesResult.fetching ||
    completionModelsResult.fetching ||
    chatModelsResult.fetching ||
    updateResult.fetching ||
    resetResult.fetching;

  const completionModels: Model[] = completionModelsResult.data?.availableModels || []
  const chatModels: Model[] = chatModelsResult.data?.availableModels || []

  return (
    <ProfileCard
      title="AI Model Preferences"
      description="Choose your preferred AI models for code completion and chat features."
    >
      <div className="grid gap-4 py-4">
        <div className="grid grid-cols-4 items-center gap-4">
          <label htmlFor="completion-model" className="text-right">
            Code Completion
          </label>
          <Select
            value={selectedCompletionModel}
            onValueChange={setSelectedCompletionModel}
            disabled={isLoading}
          >
            <SelectTrigger className="col-span-3">
              <SelectValue placeholder="System Default" />
            </SelectTrigger>
            <SelectContent id="completion-model">
              <SelectItem value="">System Default</SelectItem>
              {completionModels.map(model => (
                <SelectItem key={model.modelName} value={model.modelName}>
                  {model.modelName}
                  {model.description && (
                    <span className="ml-2 text-xs text-muted-foreground">
                      ({model.description})
                    </span>
                  )}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="grid grid-cols-4 items-center gap-4">
          <label htmlFor="chat-model" className="text-right">
            Chat Model
          </label>
          <Select
            value={selectedChatModel}
            onValueChange={setSelectedChatModel}
            disabled={isLoading}
          >
            <SelectTrigger className="col-span-3">
              <SelectValue placeholder="System Default" />
            </SelectTrigger>
            <SelectContent id="chat-model">
              <SelectItem value="">System Default</SelectItem>
              {chatModels.map(model => (
                <SelectItem key={model.modelName} value={model.modelName}>
                  {model.modelName}
                  {model.description && (
                    <span className="ml-2 text-xs text-muted-foreground">
                      ({model.description})
                    </span>
                  )}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
      <Separator />
      <div className="flex justify-end gap-2 p-4">
        <Button variant="outline" onClick={handleReset} disabled={isLoading}>
          Reset to Default
        </Button>
        <Button onClick={handleSubmit} disabled={isLoading}>
          Save Preferences
        </Button>
      </div>
    </ProfileCard>
  )
}